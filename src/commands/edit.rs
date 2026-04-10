use crate::config::{RepoConfig, UserConfig};
use crate::utils::find_editor;

pub fn execute(software_name: &str) -> Result<(), String> {
    let user_config = UserConfig::load()?;
    let repo_config = RepoConfig::from_user_cfg(&user_config)?;

    // 查找软件配置
    let software = repo_config
        .software
        .get(software_name)
        .ok_or_else(|| format!("未找到软件配置: {}", software_name))?;

    // 使用仓库路径
    let repo_path = std::path::PathBuf::from(&user_config.repo_path).join(&software.src_path);

    if !repo_path.exists() {
        return Err(format!("仓库路径不存在: {}", repo_path.display()));
    }

    // 查找编辑器
    let editor = find_editor(user_config.editor.as_deref())
        .ok_or_else(|| "未找到可用的编辑器 (zed, code, nvim, vim, vi)".to_string())?;

    println!("使用 {} 打开 {}", editor, repo_path.display());

    // 打开编辑器
    let status = std::process::Command::new(&editor)
        .arg(&repo_path)
        .status()
        .map_err(|e| format!("启动编辑器失败: {}", e))?;

    if !status.success() {
        return Err("编辑器退出异常".to_string());
    }

    Ok(())
}
