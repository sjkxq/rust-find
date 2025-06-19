use clap::Parser;
use log::{info, debug};
use anyhow::{Result, Context};
use rust_find::find::{find_files, FindOptions};

/// A Rust implementation of the Linux find command
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Search paths (default: current directory)
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// Maximum depth to search
    #[arg(long, value_name = "NUM")]
    max_depth: Option<usize>,

    /// Follow symbolic links
    #[arg(short = 'L', long)]
    follow_links: bool,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
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
    
    let options = FindOptions {
        max_depth: args.max_depth,
        follow_links: args.follow_links,
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