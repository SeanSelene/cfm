use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    path::{self, Path, PathBuf},
};
use tabled::{builder::Builder, settings::Style};
use thiserror::Error;

use crate::utils::{self, expand_path};

/// 链接模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LinkMode {
    #[default]
    Soft,
    Hard,
    Cp,
}

impl fmt::Display for LinkMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            LinkMode::Soft => "soft",
            LinkMode::Hard => "hard",
            LinkMode::Cp => "cp",
        };
        write!(f, "{}", s)
    }
}

/// 软件配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 名称
    pub name: String,
    /// 仓库中的路径
    pub src_path: String,
    /// 链接模式
    #[serde(default)]
    pub link_mode: LinkMode,
    /// Unix 系统配置路径 (Linux 和 macOS)
    pub dest_path_unix: Option<String>,
    /// Windows 配置路径
    pub dest_path_win: Option<String>,
    /// macOS 特定配置路径
    pub dest_path_mac: Option<String>,
    /// 通用配置路径
    pub dest_path: Option<String>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("链接不匹配: 源路径目标路径不一致")]
    LinkNotMatch(PathBuf, PathBuf),
    #[error("目标路径已存在: {0}")]
    DestExist(PathBuf),
    #[error("源路径不存在")]
    SrcNotExist(PathBuf),
    #[error("目标路径配置缺失")]
    DestConfigMiss,
    #[error("暂不支持软连接文件")]
    SoftFile,
    #[error("仅能硬链接文件")]
    HardDir,
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

impl ConfigError {
    pub fn is_like_exist(&self) -> bool {
        matches!(
            self,
            ConfigError::DestExist(_)
                | ConfigError::DestConfigMiss
                | ConfigError::LinkNotMatch(_, _)
        )
    }
}

impl AppConfig {
    /// 获取当前平台的配置路径
    pub fn get_config_path(&self) -> Option<String> {
        self.get_dest_path().cloned()
    }
    pub fn get_dest_path(&self) -> Option<&String> {
        // 优先匹配平台特定字段
        let target = match () {
            #[cfg(target_os = "macos")]
            _ => self.dest_path_mac.as_ref().or(self.dest_path_unix.as_ref()),
            #[cfg(all(unix, not(target_os = "macos")))]
            _ => &self.dest_path_unix.as_ref(),
            #[cfg(windows)]
            _ => &self.dest_path_win.as_ref(),
            #[cfg(not(any(unix, windows)))] // 兜底其他系统
            _ => &None,
        };
        // 如果特定平台字段是 None，则回退到通用 dest_path
        target.or(self.dest_path.as_ref())
    }
    pub fn get_dest_path_buf(&self) -> Option<PathBuf> {
        self.get_dest_path().map(|p| utils::expand_path(p).into())
    }
    fn common_check(&self, repo_path: impl AsRef<Path>) -> Result<(PathBuf, PathBuf), ConfigError> {
        let src_path = self.get_src_path_buf(repo_path.as_ref());
        if !src_path.exists() {
            return Err(ConfigError::SrcNotExist(src_path.to_path_buf()));
        }
        if src_path.is_file() && self.link_mode == LinkMode::Soft {
            return Err(ConfigError::SoftFile);
        }
        if src_path.is_dir() && self.link_mode == LinkMode::Hard {
            return Err(ConfigError::HardDir);
        }
        let Some(dest_path) = self.get_dest_path_buf() else {
            return Err(ConfigError::DestConfigMiss);
        };
        Ok((src_path, dest_path))
    }
    pub fn get_src_path_buf(&self, repo_path: &Path) -> PathBuf {
        repo_path.join(utils::expand_path(&self.src_path)) // 拼接上仓库路径
    }
    pub fn pre_check(
        &self,
        repo_path: impl AsRef<Path>,
    ) -> Result<(PathBuf, PathBuf), ConfigError> {
        let (src_path, dest_path) = self.common_check(repo_path)?;
        // 检查目标路径是否存在
        if !dest_path.exists() {
            return Ok((src_path.to_path_buf(), dest_path));
        }
        Err(ConfigError::DestExist(dest_path.to_path_buf()))
    }
    pub fn after_check(&self, repo_path: impl AsRef<Path>) -> Result<(), String> {
        let (src_path, dest_path) = self.common_check(repo_path).map_err(|e| e.to_string())?;
        if !dest_path.exists() {
            return Err("配置文件不存在".into());
        }
        // 软链接如果是一致的就不用报错
        if self.link_mode == LinkMode::Soft && dest_path.is_symlink() {
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
}

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
                let config_path = sw.get_config_path()?;
                let path = PathBuf::from(expand_path(&config_path));
                path.symlink_metadata().ok().map(|_| (sw.name.clone(), path))
            })
            .collect()
    }

    pub fn print(&self, repo_path: impl AsRef<Path>) {
        let mut builder = Builder::default();
        builder.push_record(["名称", "链接模式", "状态", "源路径", "目标路径"]);
        for sw in &self.apps {
            let dest_path = sw.get_dest_path().map(|i| i.as_str()).unwrap_or("");
            let link_mode = sw.link_mode.to_string();
            let status = match sw.after_check(&repo_path) {
                Ok(_) => "✅".into(),
                Err(e) => format!("❌ {}", e),
            };
            builder.push_record([&sw.name, &link_mode, &status, &sw.src_path, dest_path]);
        }
        let mut table = builder.build();
        table.with(Style::modern());
        println!("{table}");
    }
}

/// 用户配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// 目标路径
    pub repo_path: String,
    /// 编辑器
    #[serde(default)]
    pub editor: Option<String>,
}

impl UserConfig {
    /// 获取用户配置文件路径
    pub fn config_path() -> std::path::PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| {
            let dir = if cfg!(windows) { "%AppData%" } else { "~/.config" };
            std::path::PathBuf::from(dir)
        });
        path.push("cfm/config.toml");
        path
    }

    /// 加载用户配置
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path();
        if !path.exists() {
            return Err("配置文件不存在，请先执行 cfm init".to_string());
        }

        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))
    }

    /// 保存用户配置
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();

        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| format!("序列化配置失败: {}", e))?;

        std::fs::write(&path, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

        Ok(())
    }
}
