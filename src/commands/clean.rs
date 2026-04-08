use crate::config::{RepoConfig, UserConfig};
use crate::utils::{self, remove_link_target};

pub fn execute(force: bool) -> Result<(), String> {
    let user_config = UserConfig::load()?;
    let repo_config = RepoConfig::from_user_cfg(&user_config)?;
    let target_path = std::path::Path::new(&user_config.target_path);

    // 收集要删除的路径
    let mut paths_to_delete = repo_config.get_apply_files();

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
    if !force && utils::confirm("确定删除").is_err() {
        return Ok(());
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
