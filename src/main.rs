use clap::Parser;
use log::{info, debug};
use anyhow::{Result, Context};
use rust_find::find::{find_files, FindOptions};

/// Linux find 命令的 Rust 实现
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 搜索路径（默认：当前目录）
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// 最大搜索深度
    #[arg(long, value_name = "NUM")]
    max_depth: Option<usize>,

    /// 跟随符号链接
    #[arg(short = 'L', long)]
    follow_links: bool,

    /// 启用调试日志
    #[arg(short, long)]
    debug: bool,

    /// 输出绝对路径
    #[arg(long)]
    absolute: bool,

    /// 输出相对路径（相对于当前目录）
    #[arg(long, conflicts_with = "absolute")]
    relative: bool,

    /// 按文件名模式匹配 (支持通配符，可多次指定)
    #[arg(short = 'n', long, conflicts_with = "iname")]
    name: Vec<String>,

    /// 不区分大小写的文件名匹配 (支持通配符，可多次指定)
    #[arg(short = 'i', long = "iname", conflicts_with = "name")]
    iname: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger
    env_logger::Builder::new()
        .filter_level(if args.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    info!("Starting rust-find");
    
    let (name_patterns, ignore_case) = if !args.name.is_empty() {
        (args.name, false)
    } else if !args.iname.is_empty() {
        (args.iname, true)
    } else {
        (Vec::new(), false)
    };

    let options = FindOptions {
        parallel: false,
        max_depth: args.max_depth,
        follow_links: args.follow_links,
        absolute_path: args.absolute,
        relative_path: args.relative,
        name_patterns,
        ignore_case,
    };

    for path in &args.paths {
        debug!("Searching in path: {}", path);
        let results = find_files(path, &options)
            .with_context(|| format!("Failed to search in path: {}", path))?;

        // Print found files
        for file in results {
            println!("{}", file.display());
        }
    }

    Ok(())
}