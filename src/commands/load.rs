use super::apply::apply;
#[cfg(windows)]
use crate::config::LinkMode;
use crate::config::{RepoConfig, UserConfig};
use crate::utils::{self, expand_path};

/// 从 Git URL 中提取仓库名
fn extract_repo_name(url: &str) -> String {
    // 移除末尾的 .git
    let url = url.strip_suffix(".git").unwrap_or(url);
    // 获取最后一个路径段
    url.rsplit('/').next().unwrap_or(url).to_string()
}

pub fn execute(repo_url: &str, repo_path: Option<&str>) -> Result<(), String> {
    let is_repo = utils::is_git_repo(repo_url);
    // 获取目标文件夹 (dotfiles 本地路径或者仓库克隆下来要保存的路径)
    let repo_path = match repo_path {
        // 有第二个参数说明是 clone 的情况
        Some(path) => {
            if !is_repo {
                return Err("非法的 git 仓库路径".into());
            };
            expand_path(path)
        }
        None => {
            if is_repo {
                let repo_name = extract_repo_name(repo_url);
                let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
                home.join(&repo_name).to_string_lossy().into_owned()
            } else {
                // load 本地仓库或目录
                utils::expand_path(repo_url)
            }
        }
    };
    let configs_path = std::path::Path::new(&repo_path);
    if is_repo {
        println!("正在克隆 {} 到 {}\n", repo_url, repo_path);
        // 检查目标目录是否已存在
        if configs_path.exists() {
            return Err(format!("目标目录已存在: {}", repo_path));
        }
        // 克隆仓库
        let status = std::process::Command::new("git")
            .args(["clone", repo_url, &repo_path])
            .status()
            .map_err(|e| format!("执行 git clone 失败: {}", e))?;
        if !status.success() {
            return Err("克隆仓库失败".to_string());
        }
    } else if !configs_path.exists() {
        return Err(format!("目录不存在: {}", repo_path));
    }

    // 读取仓库中的 cfm.toml
    let repo_config = RepoConfig::from_path(configs_path)?;

    // 先保存用户配置，以便提权后的进程可以正常加载
    let user_config = UserConfig { repo_path: repo_path.clone(), editor: None };
    user_config.save()?;

    // Windows 下若存在 Soft 链接，需要管理员权限再执行 apply
    #[cfg(windows)]
    {
        let needs_admin = repo_config.apps.iter().any(|app| app.link_mode == LinkMode::Soft);
        if needs_admin && !utils::is_elevated() {
            utils::elevate(&["apply"])?;
            println!("\n初始化完成，配置已保存到 {}", UserConfig::config_path().display());
            return Ok(());
        }
    }

    // 处理软件配置
    apply(repo_config.apps.iter(), false, configs_path)?;

    println!("\n初始化完成，配置已保存到 {}", UserConfig::config_path().display());
    println!();
    println!("软件列表：");
    // 显示软件列表
    repo_config.print(&user_config.repo_path);
    Ok(())
}
