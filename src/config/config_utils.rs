use std::fs;
use std::path::{Path, PathBuf};

use super::{AppConfig, ConfigError, LinkMode};
use crate::utils;

pub fn common_check(
    config: &AppConfig,
    repo_path: impl AsRef<Path>,
) -> Result<(PathBuf, PathBuf), ConfigError> {
    let repo_path = repo_path.as_ref();
    let src_path = repo_path.join(&config.src_path);
    if !src_path.exists() {
        return Err(ConfigError::SrcNotExist(src_path.to_path_buf()));
    }
    if src_path.is_dir() && config.link_mode == LinkMode::Hard {
        return Err(ConfigError::HardDir);
    }
    let dest_path = utils::expand_path(&config.dest_path);
    Ok((src_path, PathBuf::from(dest_path)))
}

pub fn pre_check(
    config: &AppConfig,
    repo_path: impl AsRef<Path>,
) -> Result<(PathBuf, PathBuf), ConfigError> {
    let (src_path, dest_path) = common_check(config, repo_path)?;
    // 检查目标路径是否存在
    if !dest_path.exists() {
        return Ok((src_path, dest_path));
    }
    Err(ConfigError::DestExist { dest_path, src_path })
}

pub fn after_check(config: &AppConfig, repo_path: impl AsRef<Path>) -> Result<(), String> {
    let (src_path, dest_path) = common_check(config, repo_path).map_err(|e| e.to_string())?;
    if !dest_path.exists() {
        return Err("配置文件不存在".into());
    }
    // 软链接如果是一致的就不用报错
    if config.link_mode == LinkMode::Soft && dest_path.is_symlink() {
        let dest_path_to = match fs::read_link(dest_path) {
            Ok(path) => path,
            Err(e) => return Err(e.to_string()),
        };
        if dest_path_to != src_path {
            return Err("链接不一致".into());
        }
    }
    Ok(())
}
