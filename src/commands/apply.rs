use std::{collections::HashSet, fs, path::PathBuf};

use crate::{
    config::{AppConfig, ConfigError, LinkMode, RepoConfig, UserConfig},
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
        match app.pre_check(repo_path) {
            Ok((src, dest)) => {
                to_handle.push((&app.name, &app.link_mode, src, dest));
            }
            Err(e) => match e {
                ConfigError::DestExist(path_buf) => {
                    exist.push(path_buf);
                    to_handle.push((
                        &app.name,
                        &app.link_mode,
                        app.get_src_path_buf(repo_path),
                        app.get_dest_path_buf().unwrap(), // SAFETY: 已经在 check_dest_path 中检查过了
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
            return Err(format!("删除 {file_path:?} 失败: {e}"));
        }
    }
    for (name, mode, src, dest) in to_handle {
        let res = match mode {
            LinkMode::Soft => utils::soft_link_dir(src, dest),
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
    let apps = repo_config.apps.iter().filter(|app| is_empty || names.contains(&app.name));
    apply(apps, false, &user_config.repo_path)?;
    repo_config.print(&user_config.repo_path);
    Ok(())
}
