use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// 展开路径中的环境变量和 ~
pub fn expand_path(input: &str) -> String {
    let expanded = shellexpand::full(input).unwrap_or_else(|_| input.into());
    if cfg!(windows) && expanded.contains('/') {
        expanded.replace("/", "\\")
    } else {
        expanded.into_owned()
    }
}

/// 查找可用的编辑器
pub fn find_editor(configured_editor: Option<&str>) -> Option<String> {
    // 如果配置了编辑器，直接使用
    if let Some(editor) = configured_editor
        && which::which(editor).is_ok()
    {
        return Some(editor.to_string());
    }

    // 按优先级查找
    let editors = ["zed", "code", "nvim", "vim", "vi"];
    for editor in editors {
        if which::which(editor).is_ok() {
            return Some(editor.to_string());
        }
    }

    None
}

/// 创建目录的软连接(unix)或junction(windows)
/// 此方法没有确认源存在，也没有删除目标路径，也没有确认是文件夹
pub fn soft_link_dir<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    let original = original.as_ref();
    let link = link.as_ref();
    // 确保父级存在
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match () {
        #[cfg(windows)]
        _ => junction::create(original, link),
        #[cfg(not(windows))]
        _ => std::os::unix::fs::symlink(original, link),
    }
}

/// 删除链接目标（正确处理 Junction/符号链接）
pub fn remove_link_target(path: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        // Windows: Junction 需要使用 junction::delete，否则会删除源目录内容
        // 使用 symlink_metadata 检查是否是 reparse point (junction)
        if let Ok(metadata) = path.symlink_metadata() {
            use std::os::windows::fs::MetadataExt;
            // FILE_ATTRIBUTE_REPARSE_POINT = 0x400
            if metadata.file_attributes() & 0x400 != 0 {
                // 这是一个 reparse point (junction 或 symlink)
                junction::delete(path).map_err(|e| format!("删除目录连接失败: {}", e))?;
                return Ok(());
            }
        }
    }

    // 符号链接或普通文件/目录
    if path.is_dir() {
        // 先尝试 remove_dir，如果目录非空再用 remove_dir_all
        if std::fs::remove_dir(path).is_err() {
            std::fs::remove_dir_all(path).map_err(|e| format!("删除目录失败: {}", e))?;
        }
        Ok(())
    } else {
        std::fs::remove_file(path).map_err(|e| format!("删除文件失败: {}", e))
    }
}

pub fn copy_dir_recursive<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // 用 symlink_metadata 获取条目本身的类型，不跟随符号链接
        let metadata = fs::symlink_metadata(&src_path)?;
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            // TODO: 符号链接的处理
            todo!()
        } else if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

pub fn is_git_repo(path: &str) -> bool {
    let starts = ["git@", "http://", "https://", "ssh://", "git://"];
    starts.iter().any(|s| path.starts_with(s))
}

pub fn confirm(tip: &str) -> Result<(), String> {
    print!("{tip}[y/N] ");
    io::stdout().flush().map_err(|e| format!("刷新输出失败: {}", e))?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| format!("读取输入失败: {}", e))?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        return Err("已取消".into());
    };
    Ok(())
}
