use crate::config::RepoConfig;
use crate::utils::{check_initialized, copy_path, create_hard_link, create_soft_link, expand_path};

pub fn execute() -> Result<(), String> {
    let user_config = check_initialized()?;

    let target_path = std::path::Path::new(&user_config.target_path);

    // 拉取主仓库更新
    println!("拉取仓库更新...");

    let status = std::process::Command::new("git")
        .current_dir(target_path)
        .args(["pull"])
        .status()
        .map_err(|e| format!("执行 git pull 失败: {}", e))?;

    if !status.success() {
        return Err("拉取更新失败".to_string());
    }

    // 读取仓库配置
    let config_path = target_path.join("cfm.toml");
    if !config_path.exists() {
        return Err("仓库中未找到 cfm.toml 配置文件".to_string());
    }

    let config_content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let repo_config: RepoConfig =
        toml::from_str(&config_content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 创建链接
    for (name, software) in &repo_config.software {
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

            match result {
                Ok(()) => println!("已链接 {} -> {}", name, dst.display()),
                Err(e) => println!("链接 {} 失败: {}", name, e),
            }
        } else {
            println!("跳过 {}: 未配置当前平台的路径", name);
        }
    }

    println!("同步完成");

    Ok(())
}
