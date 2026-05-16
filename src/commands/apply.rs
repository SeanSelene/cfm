use std::{collections::HashSet, fs, path::PathBuf};

use crate::{
    config::{self, AppConfig, ConfigError, LinkMode, RepoConfig, UserConfig},
    utils::{self, confirm},
};

pub fn apply<'a>(
    apps: impl IntoIterator<Item = &'a AppConfig>,
    force: bool,
    repo_path: impl AsRef<std::path::Path>,
) -> Result<(), String> {
    let repo_path = repo_path.as_ref();
    let mut exist = Vec::new();
    let mut to_handle: Vec<(&str, &LinkMode, PathBuf, PathBuf)> = Vec::new();
    for app in apps {
        match config::pre_check(app, repo_path) {
            Ok((src, dest)) => {
                to_handle.push((&app.name, &app.link_mode, src, dest));
            }
            Err(e) => match e {
                ConfigError::DestExist { dest_path, src_path } => {
                    exist.push(dest_path.clone());
                    to_handle.push((
                        &app.name,
                        &app.link_mode,
                        src_path,
                        dest_path, // SAFETY: 已经在 check_dest_path 中检查过了
                    ));
                }
                ConfigError::SrcNotExist(pb) => {
                    println!("{} 的源路径 {} 不存在, 将跳过", app.name, pb.display());
                }
                _ => return Err(format!("检查 {} 的配置出错: {e}", app.name)),
            },
        }
    }
    if !exist.is_empty() && !force {
        let msg = exist.iter().map(|i| i.to_string_lossy()).collect::<Vec<_>>().join("\n");
        let tip = format!("以下路径已存在:\n{msg}\n是否覆盖? ");
        confirm(&tip)?;
    };
    // 删除已经存在的目录
    for file_path in exist {
        let res = if file_path.is_file() {
            fs::remove_file(&file_path)
        } else {
            fs::remove_dir_all(&file_path)
        };
        if let Err(e) = res {
            match e.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => return Err(format!("删除 {file_path:?} 失败: {e}")),
            }
        }
    }
    for (name, mode, src, dest) in to_handle {
        let res = match mode {
            LinkMode::Soft => utils::soft_link(src, dest),
            LinkMode::Hard => fs::hard_link(src, dest),
            LinkMode::Cp => utils::copy_dir_recursive(src, dest),
        };
        if let Err(e) = res {
            return Err(format!("处理 {name} 失败: {e}"));
        }
    }
    Ok(())
}

pub fn execute(names: Option<Vec<String>>) -> Result<(), String> {
    let user_config = UserConfig::load()?;
    let repo_config = RepoConfig::from_user_cfg(&user_config)?;
    let names: HashSet<String> = names.map(|n| n.into_iter().collect()).unwrap_or_default();
    let is_empty = names.is_empty();

    // Windows 下创建符号链接需要管理员权限，若有 Soft 链接则尝试提权重启自身
    #[cfg(windows)]
    {
        let needs_admin = repo_config
            .apps
            .iter()
            .filter(|app| is_empty || names.contains(&app.name))
            .any(|app| app.link_mode == LinkMode::Soft);
        if needs_admin && !utils::is_elevated() {
            let args: Vec<String> = std::env::args().skip(1).collect();
            let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
            utils::elevate(&args_refs)?;
            return Ok(());
        }
    }

    let apps = repo_config.apps.iter().filter(|app| is_empty || names.contains(&app.name));
    let mut apps = apps.peekable();
    if apps.peek().is_none() {
        return Err("没有需要处理的应用，请检查配置或参数".to_string());
    }
    apply(apps, false, &user_config.repo_path)?;
    println!("\n已完成，所有应用配置如下：");
    repo_config.print(&user_config.repo_path);
    Ok(())
}
