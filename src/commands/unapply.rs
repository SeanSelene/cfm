use std::collections::HashSet;
use std::fs;
use std::path;

use crate::config::RepoConfig;
use crate::utils;

pub fn execute(names: Option<HashSet<String>>, force: bool) -> Result<(), String> {
    let repo_config = RepoConfig::from_user_cfg_file()?;
    let no_filter = names.is_none();
    let names = names.unwrap_or_default();
    let paths_to_delete = repo_config
        .get_apply_files()
        .into_iter()
        .filter(|(name, _)| no_filter || !names.contains(name))
        .collect::<Vec<(String, path::PathBuf)>>();
    if paths_to_delete.is_empty() {
        println!("没有文件需要清理");
        return Ok(());
    }
    println!("以下目录/文件将被删除:");
    for (name, path) in &paths_to_delete {
        println!("  [{}] {}", name, path.display());
    }
    // 确认删除
    if !force && utils::confirm("确定删除").is_err() {
        return Ok(());
    }
    // 执行删除
    for (_name, p) in &paths_to_delete {
        match if p.is_dir() { fs::remove_dir_all(p) } else { fs::remove_file(p) } {
            Ok(()) => println!("已删除: {}", p.display()),
            Err(e) => println!("删除 {} 失败: {}", p.display(), e),
        };
    }
    Ok(())
}
