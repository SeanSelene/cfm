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

/// 创建软链接
///
/// - Unix: 使用 `symlink` 同时支持文件和目录
/// - Windows: 根据源是文件或目录分别使用 `symlink_file` / `symlink_dir`
///   （需要管理员权限或开启开发者模式）
///
/// 此方法不验证源是否存在，也不会删除目标路径。
pub fn soft_link<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    let original = original.as_ref();
    let link = link.as_ref();
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }
    #[cfg(windows)]
    {
        if original.is_dir() {
            std::os::windows::fs::symlink_dir(original, link)
        } else {
            std::os::windows::fs::symlink_file(original, link)
        }
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(original, link)
    }
}

/// 删除链接目标（正确处理 Junction / 符号链接 / 普通文件目录）
pub fn remove_link_target(path: &std::path::Path) -> Result<(), String> {
    let metadata = match path.symlink_metadata() {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("读取元数据失败: {}", e)),
    };
    let file_type = metadata.file_type();

    #[cfg(windows)]
    {
        // Windows: 兼容旧版本创建的 Junction（reparse point 但不是 symlink）
        use std::os::windows::fs::MetadataExt;
        let is_reparse = metadata.file_attributes() & 0x400 != 0;
        if is_reparse && !file_type.is_symlink() {
            return junction::delete(path).map_err(|e| format!("删除目录连接失败: {}", e));
        }
    }

    if file_type.is_symlink() {
        // 符号链接：根据指向类型分别处理（不会跟随删除目标内容）
        if path.is_dir() {
            std::fs::remove_dir(path).map_err(|e| format!("删除目录链接失败: {}", e))
        } else {
            std::fs::remove_file(path).map_err(|e| format!("删除文件链接失败: {}", e))
        }
    } else if file_type.is_dir() {
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

/// Windows 下检查当前进程是否拥有管理员权限。
#[cfg(windows)]
pub fn is_elevated() -> bool {
    use std::mem;
    use std::ptr;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{HANDLE, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};

    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        CloseHandle(token);
        ok != 0 && elevation.TokenIsElevated != 0
    }
}

/// Windows 下以管理员权限重新启动当前可执行文件，并等待其完成。
///
/// 提权后的进程会在新的控制台窗口中运行，因此操作输出在新窗口可见。
#[cfg(windows)]
pub fn elevate(args: &[&str]) -> Result<(), String> {
    use std::mem;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::GetExitCodeProcess;
    use winapi::um::shellapi::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
    use winapi::um::synchapi::WaitForSingleObject;
    use winapi::um::winbase::INFINITE;
    use winapi::um::winuser::SW_SHOWNORMAL;

    let exe = std::env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {}", e))?;

    // 简单引号转义：仅当包含空格时加上双引号
    let args_str = args
        .iter()
        .map(|a| if a.contains(' ') { format!("\"{}\"", a) } else { (*a).to_string() })
        .collect::<Vec<_>>()
        .join(" ");

    let exe_wide: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let verb_wide: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();
    let args_wide: Vec<u16> = args_str.encode_utf16().chain(std::iter::once(0)).collect();

    println!("检测到软链接配置，需要管理员权限。请在弹出的 UAC 窗口中确认...");

    unsafe {
        let mut sei: SHELLEXECUTEINFOW = mem::zeroed();
        sei.cbSize = mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        sei.fMask = SEE_MASK_NOCLOSEPROCESS;
        sei.lpVerb = verb_wide.as_ptr();
        sei.lpFile = exe_wide.as_ptr();
        sei.lpParameters = args_wide.as_ptr();
        sei.nShow = SW_SHOWNORMAL;

        if ShellExecuteExW(&mut sei) == 0 {
            return Err("提权失败：用户取消或系统拒绝".into());
        }

        if sei.hProcess.is_null() {
            return Ok(());
        }

        WaitForSingleObject(sei.hProcess, INFINITE);
        let mut exit_code: u32 = 0;
        GetExitCodeProcess(sei.hProcess, &mut exit_code);
        CloseHandle(sei.hProcess);

        if exit_code != 0 {
            return Err(format!("管理员进程异常退出，退出码: {}", exit_code));
        }
    }

    Ok(())
}
