use serde::{Deserialize, Serialize};

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
