use crate::config::RepoConfig;
use crate::utils::{check_initialized, expand_path};

pub fn execute(software: Option<&str>) -> Result<(), String> {
    let user_config = check_initialized()?;

    let target_path = std::path::Path::new(&user_config.target_path);

    // 读取仓库配置
    let config_path = target_path.join("cfm.toml");
    if !config_path.exists() {
        return Err("仓库中未找到 cfm.toml 配置文件".to_string());
    }

    let config_content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let repo_config: RepoConfig =
        toml::from_str(&config_content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 确定要同步的软件
    let software_to_sync: Vec<_> = if let Some(name) = software {
        vec![(
            name.to_string(),
            repo_config
                .software
                .get(name)
                .ok_or_else(|| format!("未找到软件配置: {}", name))?
                .clone(),
        )]
    } else {
        repo_config
            .software
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    };

    // 同步配置文件
    for (name, software) in &software_to_sync {
        if let Some(config_path) = software.get_config_path() {
            let src = std::path::PathBuf::from(expand_path(&config_path));
            let dst = target_path.join(&software.repo_path);

            if !src.exists() {
                println!("跳过 {}: 配置路径不存在 {}", name, src.display());
                continue;
            }

            // 复制配置到仓库
            if src.is_dir() {
                // 先删除目标目录
                if dst.exists() {
                    std::fs::remove_dir_all(&dst).map_err(|e| format!("删除目录失败: {}", e))?;
                }
                // 复制整个目录
                copy_dir_all(&src, &dst)?;
            } else {
                std::fs::copy(&src, &dst).map_err(|e| format!("复制文件失败: {}", e))?;
            }

            println!("已同步 {} -> {}", src.display(), dst.display());
        }
    }

    // 提交主仓库更改
    println!("提交更改...");

    let status = std::process::Command::new("git")
        .current_dir(target_path)
        .args(["add", "."])
        .status()
        .map_err(|e| format!("执行 git add 失败: {}", e))?;

    if !status.success() {
        return Err("添加文件失败".to_string());
    }

    let _status = std::process::Command::new("git")
        .current_dir(target_path)
        .args(["commit", "-m", "Update config files"])
        .status()
        .map_err(|e| format!("执行 git commit 失败: {}", e))?;

    // 即使没有更改也继续推送
    let status = std::process::Command::new("git")
        .current_dir(target_path)
        .args(["push"])
        .status()
        .map_err(|e| format!("执行 git push 失败: {}", e))?;

    if !status.success() {
        return Err("推送失败".to_string());
    }

    println!("推送完成");

    Ok(())
}

/// 递归复制目录
fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;

    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let ty = entry
            .file_type()
            .map_err(|e| format!("获取文件类型失败: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }

    Ok(())
}
