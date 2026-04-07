use super::list::print_software_list;
use crate::config::{RepoConfig, UserConfig};
use crate::utils::{copy_path, create_hard_link, create_soft_link, expand_path};

/// 从 Git URL 中提取仓库名
fn extract_repo_name(url: &str) -> String {
    // 移除末尾的 .git
    let url = url.strip_suffix(".git").unwrap_or(url);
    // 获取最后一个路径段
    url.rsplit('/').next().unwrap_or(url).to_string()
}

pub fn execute(repo_url: &str, target_path: Option<&str>) -> Result<(), String> {
    // 如果未指定目标路径，使用 ~/{仓库名}
    let target = match target_path {
        Some(path) => expand_path(path),
        None => {
            let repo_name = extract_repo_name(repo_url);
            let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
            home.join(&repo_name).to_string_lossy().into_owned()
        }
    };

    println!("克隆仓库 {} 到 {}\n", repo_url, target);

    // 检查目标目录是否已存在
    let target_path = std::path::Path::new(&target);
    if target_path.exists() {
        return Err(format!("目标目录已存在: {}", target));
    }

    // 克隆仓库
    let status = std::process::Command::new("git")
        .args(["clone", repo_url, &target])
        .status()
        .map_err(|e| format!("执行 git clone 失败: {}", e))?;

    if !status.success() {
        return Err("克隆仓库失败".to_string());
    }

    // 读取仓库中的 cfm.toml
    let config_path = target_path.join("cfm.toml");
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
            let src = target_path.join(&software.repo_path);
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
    let user_config = UserConfig {
        repo_dir: repo_url.to_string(),
        target_path: target.clone(),
        editor: None,
    };

    user_config.save()?;

    // 显示软件列表
    print_software_list(&repo_config, target_path);

    println!(
        "\n初始化完成，配置已保存到 {}",
        UserConfig::config_path().display()
    );

    Ok(())
}
