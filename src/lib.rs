//! cfm - 跨平台配置文件管理工具
//!
//! 通过 Git 仓库统一管理各类软件的配置文件。

pub mod commands;
pub mod config;
mod utils;

pub use commands::{clean, edit, init, list, pull, push};
