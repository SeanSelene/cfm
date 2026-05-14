use serde::{Deserialize, Serialize};
use std::path::{self, Path, PathBuf};
use tabled::{builder::Builder, settings::Style};

use super::{AppConfig, UserConfig, after_check};
use crate::utils;

/// 仓库配置文件 (cfm.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub apps: Vec<AppConfig>,
}

impl RepoConfig {
    pub fn from_user_cfg_file() -> Result<Self, String> {
        let user_cfg = UserConfig::load()?;
        Self::from_user_cfg(&user_cfg)
    }

    pub fn from_user_cfg(user_cfg: &UserConfig) -> Result<Self, String> {
        Self::from_path(&user_cfg.repo_path)
    }

    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let config_path = path.as_ref().join("cfm.toml");
        if !config_path.exists() {
            return Err(format!("缺失必要文件 {}", config_path.display()));
        }
        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        toml::from_str(&config_content).map_err(|e| format!("解析配置文件失败: {}", e))
    }

    pub fn get_apply_files(&self) -> Vec<(String, path::PathBuf)> {
        self.apps
            .iter()
            .filter_map(|sw| {
                let path = PathBuf::from(utils::expand_path(&sw.dest_path));
                path.symlink_metadata().ok().map(|_| (sw.name.clone(), path))
            })
            .collect()
    }

    pub fn print(&self, repo_path: impl AsRef<Path>) {
        let mut builder = Builder::default();
        builder.push_record(["名称", "链接模式", "状态", "源路径", "目标路径"]);
        for sw in &self.apps {
            let link_mode = sw.link_mode.to_string();
            let status = match after_check(sw, &repo_path) {
                Ok(_) => "✅".into(),
                Err(e) => format!("❌ {}", e),
            };
            builder.push_record([&sw.name, &link_mode, &status, &sw.src_path, &sw.dest_path]);
        }
        let mut table = builder.build();
        table.with(Style::modern());
        println!("{table}");
    }
}
