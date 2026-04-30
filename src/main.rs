mod commands;

use clap::{Parser, Subcommand};
use commands::{md5_hash, sha256_rename};

/// 实用命令行工具集
#[derive(Parser, Debug)]
#[command(
    name = "utils",
    version,
    about = "一组实用的命令行工具",
    long_about = "一组实用的命令行工具集合，包含文件哈希计算、重命名等功能。\n将来会持续扩展更多工具。"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 计算文件 SHA256 并将文件重命名为哈希值
    #[command(name = "sha256", alias = "s")]
    Sha256(sha256_rename::Sha256RenameArgs),

    /// 计算字符串的 MD5（默认 16 位小写，取第 9-24 位）
    #[command(name = "md5", alias = "m")]
    Md5(md5_hash::Md5HashArgs),
}

fn main() {
    let cli = Cli::parse();
    let result = match &cli.command {
        Commands::Sha256(args) => sha256_rename::run(args),
        Commands::Md5(args) => md5_hash::run(args),
    };
    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
