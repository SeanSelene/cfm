use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 链接模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LinkMode {
    #[default]
    Soft,
    Hard,
    Cp,
}

/// 软件配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareConfig {
    /// 仓库中的路径
    pub repo_path: String,
    /// 链接模式
    #[serde(default)]
    pub link_mode: LinkMode,
    /// Unix 系统配置路径 (Linux 和 macOS)
    pub config_path_unix: Option<String>,
    /// Windows 配置路径
    pub config_path_win: Option<String>,
    /// macOS 特定配置路径
    pub config_path_mac: Option<String>,
    /// 通用配置路径
    pub config_path: Option<String>,
}

impl SoftwareConfig {
    /// 获取当前平台的配置路径
    pub fn get_config_path(&self) -> Option<String> {
        // 优先使用平台特定路径
        #[cfg(target_os = "macos")]
        {
            if let Some(path) = &self.config_path_mac {
                return Some(path.clone());
            }
        }

        #[cfg(unix)]
        {
            if let Some(path) = &self.config_path_unix {
                return Some(path.clone());
            }
        }

        #[cfg(windows)]
        {
            if let Some(path) = &self.config_path_win {
                return Some(path.clone());
            }
        }

        // 回退到通用路径
        self.config_path.clone()
    }
}

/// 仓库配置文件 (cfm.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    #[serde(flatten)]
    pub software: HashMap<String, SoftwareConfig>,
}

impl RepoConfig {
    pub fn from_user_cfg_file() -> Result<Self, String> {
        let user_cfg = UserConfig::load()?;
        Self::from_user_cfg(&user_cfg)
    }

    pub fn from_user_cfg(user_cfg: &UserConfig) -> Result<Self, String> {
        let target_path = std::path::Path::new(&user_cfg.target_path);
        let config_path = target_path.join("cfm.toml");
        if !config_path.exists() {
            return Err("仓库中未找到 cfm.toml 配置文件".to_string());
        }
        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        toml::from_str(&config_content).map_err(|e| format!("解析配置文件失败: {}", e))
    }
}

/// 用户配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// 仓库目录
    pub repo_dir: String,
    /// 目标路径
    pub target_path: String,
    /// 编辑器
    #[serde(default)]
    pub editor: Option<String>,
}

impl UserConfig {
    /// 获取用户配置文件路径
    pub fn config_path() -> std::path::PathBuf {
        #[cfg(unix)]
        {
            let mut path =
                dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("~/.config"));
            path.push("cfm/config.toml");
            path
        }

        #[cfg(windows)]
        {
            let mut path =
                dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("%AppData%"));
            path.push("cfm/config.toml");
            path
        }
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
