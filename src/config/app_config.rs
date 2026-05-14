use serde::{Deserialize, Serialize};

use super::LinkMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "RawAppConfig")] // 关键点：使用 try_from
pub struct AppConfig {
    pub name: String,
    pub src_path: String,
    pub link_mode: LinkMode,
    pub dest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAppConfig {
    /// 名称
    pub name: String,
    /// 仓库中的路径
    pub src_path: String,
    /// 链接模式 macos & linux 优先取 link_mode_unix
    #[serde(default)]
    pub link_mode: Option<LinkMode>,
    pub link_mode_unix: Option<LinkMode>,

    /// 目标位置： 优先目标平台字段
    pub dest_path_unix: Option<String>,
    pub dest_path_win: Option<String>,
    pub dest_path_mac: Option<String>,
    pub dest_path: Option<String>,
}

impl TryFrom<RawAppConfig> for AppConfig {
    type Error = &'static str;

    fn try_from(value: RawAppConfig) -> Result<Self, Self::Error> {
        // 处理链接模式：macos & linux 优先取 link_mode_unix
        #[cfg(target_os = "macos")]
        let link_mode = value.link_mode_unix.or(value.link_mode);

        #[cfg(target_os = "linux")]
        let link_mode = value.link_mode_unix.or(value.link_mode);

        #[cfg(target_os = "windows")]
        let link_mode = value.link_mode;

        let link_mode = link_mode
            .ok_or("link_mode is required, specify either 'link_mode' or 'link_mode_unix'")?;

        // 处理目标路径：按平台优先级选择
        // macOS: dest_path_mac > dest_path_unix > dest_path
        // Linux: dest_path_unix > dest_path
        // Windows: dest_path_win > dest_path
        #[cfg(target_os = "macos")]
        let dest_path = value.dest_path_mac.or(value.dest_path_unix).or(value.dest_path);

        #[cfg(target_os = "linux")]
        let dest_path = value.dest_path_unix.or(value.dest_path);

        #[cfg(target_os = "windows")]
        let dest_path = value.dest_path_win.or(value.dest_path);

        let dest_path = dest_path.ok_or(
            "dest_path is required, specify at least one of: 'dest_path', 'dest_path_unix', 'dest_path_win', or 'dest_path_mac'",
        )?;

        Ok(AppConfig { name: value.name, src_path: value.src_path, link_mode, dest_path })
    }
}
