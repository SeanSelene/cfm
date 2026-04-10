use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::{
    config::{ConfigError, LinkMode, RepoConfig, SoftwareConfig, UserConfig},
    utils::{self, confirm},
};

pub fn apply(
    software_map: &HashMap<&String, &SoftwareConfig>,
    force: bool,
    repo_path: impl AsRef<std::path::Path>,
) -> Result<(), String> {
    let repo_path = repo_path.as_ref();
    let mut exist = Vec::new();
    let mut to_handle: HashMap<_, _> = HashMap::new();
    for (name, software) in software_map {
        match software.check_dest_path(repo_path) {
            Ok((src, dest)) => {
                to_handle.insert(name, (&software.link_mode, src, dest));
            }
            Err(e) => match e {
                ConfigError::DestExist(path_buf) => {
                    exist.push(path_buf);
                    to_handle.insert(
                        name,
                        (
                            &software.link_mode,
                            software.get_src_path_buf(repo_path),
                            software.get_dest_path_buf().unwrap(), // SAFETY: 已经在 check_dest_path 中检查过了
                        ),
                    );
                }
                ConfigError::SrcNotExist(pb) => {
                    println!("{name} 的源路径 {} 不存在, 将跳过", pb.display());
                }
                _ => return Err(format!("检查 {name} 的配置出错: {e}")),
            },
        };
    }
    if !exist.is_empty() && !force {
        let msg = exist.iter().map(|i| i.to_string_lossy()).collect::<Vec<_>>().join("\n");
        let tip = format!("以下路径已存在:\n{msg}\n是否覆盖? \n");
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
    for (name, (mode, src, dest)) in to_handle {
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
    let software: HashMap<_, _> =
        repo_config.software.iter().filter(|(name, _)| is_empty || names.contains(*name)).collect();
    apply(&software, false, user_config.repo_path)?;
    Ok(())
}
