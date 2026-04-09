use std::io::{self, Write};
use std::path::PathBuf;

/// 展开路径中的环境变量和 ~
pub fn expand_path(input: &str) -> String {
    let expanded = shellexpand::full(input).unwrap_or_else(|_| input.into());
    if cfg!(windows) && expanded.contains('/') {
        expanded.replace("/", "\\")
    } else {
        expanded.into_owned()
    }
}

/// 将路径转换为 PathBuf
#[allow(dead_code)]
pub fn path_buf_from_str(path: &str) -> PathBuf {
    PathBuf::from(expand_path(path))
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

#[cfg(unix)]
pub use std::os::unix::fs::symlink as symlink_file;

#[cfg(windows)]
pub fn symlink_file<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
    original: P,
    link: Q,
) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(original, link)
}

#[cfg(unix)]
pub use std::fs::hard_link;

#[cfg(windows)]
pub fn hard_link<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
    original: P,
    link: Q,
) -> std::io::Result<()> {
    std::fs::hard_link(original, link)
}

/// 创建软链接
pub fn create_soft_link(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    // 如果目标已存在，先删除
    if dst.exists() || dst.symlink_metadata().is_ok() {
        remove_link_target(dst)?;
    }

    // 确保目标目录存在
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // Windows: 目录使用 Junction（不需要管理员权限），文件使用符号链接
    #[cfg(windows)]
    {
        if src.is_dir() {
            junction::create(src, dst).map_err(|e| format!("创建目录连接失败: {}", e))
        } else {
            symlink_file(src, dst).map_err(|e| format!("创建文件软链接失败: {}", e))
        }
    }

    #[cfg(not(windows))]
    {
        symlink_file(src, dst).map_err(|e| format!("创建软链接失败: {}", e))
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

/// 创建硬链接
pub fn create_hard_link(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    // 如果目标已存在，先删除
    if dst.exists() {
        std::fs::remove_file(dst).map_err(|e| format!("删除文件失败: {}", e))?;
    }

    // 确保目标目录存在
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    hard_link(src, dst).map_err(|e| format!("创建硬链接失败: {}", e))
}

/// 复制文件或目录
pub fn copy_path(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    println!("从 {} 复制到 {}", src.display(), dst.display());
    // 如果目标已存在，先删除
    if dst.exists() {
        if dst.is_dir() {
            std::fs::remove_dir_all(dst).map_err(|e| format!("删除目录失败: {}", e))?;
        } else {
            std::fs::remove_file(dst).map_err(|e| format!("删除文件失败: {}", e))?;
        }
    }

    // 确保目标目录存在
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    if src.is_dir() {
        copy_dir_all(src, dst)?;
    } else {
        std::fs::copy(src, dst).map_err(|e| format!("复制文件失败: {}", e))?;
    }

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

pub fn is_git_repo(path: &str) -> bool {
    let starts = ["git@", "http://", "https://", "ssh://", "git://"];
    starts.iter().any(|s| path.starts_with(s))
}

pub fn confirm(tip: &str) -> Result<(), String> {
    print!("{tip}? [y/N] ");
    io::stdout()
        .flush()
        .map_err(|e| format!("刷新输出失败: {}", e))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("读取输入失败: {}", e))?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        return Err("已取消".into());
    };
    Ok(())
}
