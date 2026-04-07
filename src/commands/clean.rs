use crate::config::{RepoConfig, UserConfig};
use crate::utils::{check_initialized, expand_path, remove_link_target};
use std::io::{self, Write};

pub fn execute(force: bool) -> Result<(), String> {
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

    // 收集要删除的路径
    let mut paths_to_delete: Vec<(String, std::path::PathBuf)> = Vec::new();

    for (name, software) in &repo_config.software {
        if let Some(config_path) = software.get_config_path() {
            let path = std::path::PathBuf::from(expand_path(&config_path));
            if path.exists() || path.symlink_metadata().is_ok() {
                paths_to_delete.push((name.clone(), path));
            }
        }
    }

    // 添加克隆目录
    if target_path.exists() {
        paths_to_delete.push(("克隆目录".to_string(), target_path.to_path_buf()));
    }

    // 添加配置文件目录
    let config_file = UserConfig::config_path();
    if config_file.exists() {
        paths_to_delete.push(("配置文件".to_string(), config_file));
    }

    if paths_to_delete.is_empty() {
        println!("没有需要删除的内容");
        return Ok(());
    }

    // 显示将要删除的内容
    println!("以下目录/文件将被删除:\n");
    for (name, path) in &paths_to_delete {
        println!("  [{}] {}", name, path.display());
    }
    println!();

    // 确认删除
    if !force {
        print!("确认删除? [y/N] ");
        io::stdout()
            .flush()
            .map_err(|e| format!("刷新输出失败: {}", e))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("读取输入失败: {}", e))?;

        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("已取消");
            return Ok(());
        }
    }

    // 执行删除
    for (_name, path) in &paths_to_delete {
        match remove_link_target(path) {
            Ok(()) => println!("已删除: {}", path.display()),
            Err(e) => println!("删除 {} 失败: {}", path.display(), e),
        }
    }

    // 删除配置文件所在目录（如果为空）
    if let Some(config_dir) = UserConfig::config_path().parent()
        && config_dir.exists()
        && config_dir.read_dir().is_ok_and(|mut d| d.next().is_none())
    {
        let _ = std::fs::remove_dir(config_dir);
    }

    println!("\n清理完成");

    Ok(())
}
