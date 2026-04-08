//! cfm - 跨平台配置文件管理工具
//!
//! 通过 Git 仓库统一管理各类软件的配置文件。

pub mod commands;
pub mod config;
mod utils;

pub use commands::{clean, edit, init, list, pull, push};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cfm")]
#[command(about = "配置文件管理工具", long_about = None)]
struct Cli {
    /// 显示版本信息
    #[arg(short, long)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化配置，克隆仓库并创建链接
    Init {
        /// Git 仓库地址
        repo: String,
        /// 目标路径（默认：~/{仓库名}）
        target_path: Option<String>,
    },
    /// 使用编辑器打开软件配置目录
    Edit {
        /// 软件名称
        software: String,
    },
    /// 列出所有已配置的软件
    #[command(alias = "ls")]
    List,
    /// 拉取仓库更新并重新创建链接
    Pull,
    /// 推送配置到仓库
    Push {
        /// 只推送指定软件的配置
        #[arg(short, long)]
        software: Option<String>,
    },
    /// 清理所有创建的链接、复制的文件、克隆目录和配置文件
    Clean {
        /// 跳过确认提示
        #[arg(short, long)]
        force: bool,
    },
    /// 显示版本信息
    Version,
}

/// 运行 cfm 命令行工具
pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    if cli.version {
        println!("cfm {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let Some(command) = cli.command else {
        println!("cfm {}", env!("CARGO_PKG_VERSION"));
        println!("使用 'cfm --help' 查看帮助信息");
        return Ok(());
    };

    match command {
        Commands::Init {
            repo: repo_url,
            target_path,
        } => init(&repo_url, target_path.as_deref()),
        Commands::Edit { software } => edit(&software),
        Commands::List => list(),
        Commands::Pull => pull(),
        Commands::Push { software } => push(software.as_deref()),
        Commands::Clean { force } => clean(force),
        Commands::Version => {
            println!("cfm {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
