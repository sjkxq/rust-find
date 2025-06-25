use std::time::Instant;
use anyhow::{Result, Context};
use rust_find::finder::filter::FileFilter;
use walkdir::DirEntry;
use log::{info, debug};
use clap::Parser;

use rust_find::cli::Cli;
use rust_find::finder::{Finder, filter::NameFilter};

fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 初始化日志
    env_logger::Builder::new()
        .filter_level(if cli.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    info!("开始运行 rust-find");
    let start_time = Instant::now();

    // 为每个指定的路径执行搜索
    for path in &cli.paths {
        debug!("在路径中搜索: {}", path);

        // 创建查找选项
        let options = cli.build_options();

        // 创建过滤器
        let empty_vec = Vec::new();
        let name_patterns = if !cli.name.is_empty() {
            &cli.name
        } else if !cli.iname.is_empty() {
            &cli.iname
        } else {
            &empty_vec
        };

        // 创建名称过滤器
        let name_filter = if !name_patterns.is_empty() {
            Some(NameFilter::new(&name_patterns[0])
                .with_context(|| "创建名称过滤器失败")?)
        } else {
            None
        };

        // 创建查找器并添加过滤器
        let finder = Finder::new(options);
        let finder = if let Some(filter) = name_filter {
            finder.with_filter(filter)
        } else {
            finder
        };

        // 执行搜索
        struct AlwaysTrueFilter;
        impl FileFilter for AlwaysTrueFilter {
            fn matches(&self, _: &DirEntry) -> bool {
                true
            }

            fn description(&self) -> String {
                "始终匹配所有文件".to_string()
            }
        }

        let filter = AlwaysTrueFilter;
        let results = if cli.parallel {
            finder.find_parallel(std::path::PathBuf::from(path), filter)
        } else {
            finder.find(std::path::PathBuf::from(path), filter)
        };

        // 打印结果
        for entry in results {
            println!("{}", entry.as_path().display());
        }
    }

    let elapsed = start_time.elapsed();
    info!("搜索完成，耗时 {:.2?}", elapsed);

    Ok(())
}