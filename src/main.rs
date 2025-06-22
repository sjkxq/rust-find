use std::time::Instant;
use anyhow::{Context, Result};
use log::{info, debug};
use clap::Parser;

use rust_find::cli::Cli;
use rust_find::finder::{Finder};
use rust_find::finder::filter::FilterFactory;

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

    info!("Starting rust-find");
    let start_time = Instant::now();

    // 为每个指定的路径执行搜索
    for path in &cli.paths {
        debug!("Searching in path: {}", path);

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

        let filters = FilterFactory::create_filters(
            Some(name_patterns),
            cli.ignore_case(),
            cli.absolute,
            cli.relative,
        ).with_context(|| "Failed to create filters")?;

        // 创建查找器并添加过滤器
        let mut finder = Finder::new(options);
        for filter in filters {
            finder = finder.with_filter(filter);
        }

        // 执行搜索
        let results = if cli.parallel {
            finder.find_parallel(path)
        } else {
            finder.find(path)
        }.with_context(|| format!("Failed to search in path: {}", path))?;

        // 打印结果
        for entry in results {
            println!("{}", entry.path().display());
        }
    }

    let elapsed = start_time.elapsed();
    info!("Search completed in {:.2?}", elapsed);

    Ok(())
}