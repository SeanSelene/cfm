use super::list::print_software_list;
use crate::config::{RepoConfig, UserConfig};
use crate::utils::{self, copy_path, create_hard_link, create_soft_link, expand_path};

/// 从 Git URL 中提取仓库名
fn extract_repo_name(url: &str) -> String {
    // 移除末尾的 .git
    let url = url.strip_suffix(".git").unwrap_or(url);
    // 获取最后一个路径段
    url.rsplit('/').next().unwrap_or(url).to_string()
}

pub fn execute(repo_url: &str, target_path: Option<&str>) -> Result<(), String> {
    let is_repo = utils::is_git_repo(repo_url);
    // 获取配置文件夹
    let configs_dir = match target_path {
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
                repo_url.into()
            }
        }
    };
    let configs_path = std::path::Path::new(&configs_dir);
    if is_repo {
        println!("克隆仓库 {} 到 {}\n", repo_url, configs_dir);
        // 检查目标目录是否已存在
        if configs_path.exists() {
            return Err(format!("目标目录已存在: {}", configs_dir));
        }
        // 克隆仓库
        let status = std::process::Command::new("git")
            .args(["clone", repo_url, &configs_dir])
            .status()
            .map_err(|e| format!("执行 git clone 失败: {}", e))?;
        if !status.success() {
            return Err("克隆仓库失败".to_string());
        }
    } else if !configs_path.exists() {
        return Err(format!("目录不存在: {}", configs_dir));
    }

    // 读取仓库中的 cfm.toml
    let config_path = configs_path.join("cfm.toml");
    if !config_path.exists() {
        return Err("仓库中未找到 cfm.toml 配置文件".to_string());
    }

    let config_content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let repo_config: RepoConfig =
        toml::from_str(&config_content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 处理软件配置
    for (name, software) in &repo_config.software {
        // 创建链接
        if let Some(config_path) = software.get_config_path() {
            let src = configs_path.join(&software.repo_path);
            let dst = std::path::PathBuf::from(expand_path(&config_path));

            if !src.exists() {
                println!("跳过 {}: 源路径不存在 {}", name, src.display());
                continue;
            }

            let result = match software.link_mode {
                crate::config::LinkMode::Soft => create_soft_link(&src, &dst),
                crate::config::LinkMode::Hard => create_hard_link(&src, &dst),
                crate::config::LinkMode::Cp => copy_path(&src, &dst),
            };

            if let Err(e) = result {
                println!("链接 {} 失败: {}", name, e);
            } else {
                println!("已链接 {} -> {}", name, dst.display());
            }
        }
    }

    // 保存用户配置
    let user_config = UserConfig { target_path: configs_dir.clone(), editor: None };

    user_config.save()?;

    // 显示软件列表
    print_software_list(&repo_config, configs_path);

    println!("\n初始化完成，配置已保存到 {}", UserConfig::config_path().display());

    Ok(())
}
