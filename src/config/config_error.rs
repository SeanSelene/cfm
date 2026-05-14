use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("目标路径已存在: {dest_path}")]
    DestExist { dest_path: PathBuf, src_path: PathBuf },
    #[error("源路径不存在")]
    SrcNotExist(PathBuf),
    #[error("暂不支持软连接文件")]
    SoftFile,
    #[error("仅能硬链接文件")]
    HardDir,
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}
