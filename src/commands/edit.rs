use crate::config::RepoConfig;
use crate::utils::{check_initialized, expand_path, find_editor};

pub fn execute(software_name: &str) -> Result<(), String> {
    let user_config = check_initialized()?;

    // 读取仓库配置
    let config_path = std::path::PathBuf::from(&user_config.target_path).join("cfm.toml");
    if !config_path.exists() {
        return Err("仓库中未找到 cfm.toml 配置文件".to_string());
    }

    let config_content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let repo_config: RepoConfig =
        toml::from_str(&config_content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 查找软件配置
    let software = repo_config
        .software
        .get(software_name)
        .ok_or_else(|| format!("未找到软件配置: {}", software_name))?;

    let config_path = software
        .get_config_path()
        .ok_or_else(|| format!("未配置 {} 的路径", software_name))?;

    let expanded_path = expand_path(&config_path);
    let path = std::path::Path::new(&expanded_path);

    if !path.exists() {
        return Err(format!("配置目录不存在: {}", expanded_path));
    }

    // 查找编辑器
    let editor = find_editor(user_config.editor.as_deref())
        .ok_or_else(|| "未找到可用的编辑器 (zed, code, nvim, vim, vi)".to_string())?;

    println!("使用 {} 打开 {}", editor, expanded_path);

    // 打开编辑器
    let status = std::process::Command::new(&editor)
        .arg(&expanded_path)
        .status()
        .map_err(|e| format!("启动编辑器失败: {}", e))?;

    if !status.success() {
        return Err("编辑器退出异常".to_string());
    }

    Ok(())
}
