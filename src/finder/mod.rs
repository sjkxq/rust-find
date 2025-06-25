//! 文件查找模块
//! 
//! 这个模块提供了高性能的文件系统遍历和搜索功能，
//! 包括自适应线程池管理和高效的文件过滤机制。

mod thread_pool;
pub mod options;
pub mod filter;

use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;
use rayon::prelude::*;
use log::{debug, info};

pub use thread_pool::{AdaptiveThreadPool, ThreadPoolConfig};
pub use self::options::FindOptions;
pub use self::filter::FileFilter;

/// 文件查找器
/// 
/// 提供高性能的文件系统遍历和过滤功能，使用自适应线程池进行并行处理。
#[derive(Debug)]
pub struct Finder {
    options: FindOptions,
    thread_pool: Arc<AdaptiveThreadPool>,
}

impl Finder {
    /// 创建新的文件查找器实例
    pub fn new(options: FindOptions) -> Self {
        let thread_pool_config = ThreadPoolConfig {
            max_threads: options.max_threads,
            min_threads: options.min_threads,
            dirs_per_thread: options.dirs_per_thread,
            auto_adjust: options.auto_adjust,
        };
        
        Self {
            thread_pool: Arc::new(AdaptiveThreadPool::new(thread_pool_config)),
            options,
        }
    }

    /// 添加过滤器
    pub fn with_filter<F>(self, _filter: F) -> Self
    where
        F: FileFilter + Send + Sync + 'static,
    {
        // 这里可以存储过滤器以备后用
        self
    }

    /// 在指定目录中查找符合条件的文件
    pub fn find<F>(&self, root: PathBuf, filter: F) -> Vec<PathBuf>
    where
        F: FileFilter + Send + Sync,
    {
        self.find_parallel(root, filter)
    }

    /// 并行查找文件
    pub fn find_parallel<F>(&self, root: PathBuf, filter: F) -> Vec<PathBuf>
    where
        F: FileFilter + Send + Sync,
    {
        // 首先统计目录数量以优化线程池大小
        let dir_count = self.count_directories(&root);
        self.thread_pool.update_directory_count(dir_count);
        
        info!("Starting search in {} with {} directories", root.display(), dir_count);
        debug!("Adjusting thread pool size...");
        
        let thread_count = self.thread_pool.adjust_thread_count();
        info!("Using {} threads for search", thread_count);

        // 创建文件遍历器
        let walker = WalkDir::new(root)
            .follow_links(self.options.follow_links)
            .max_depth(self.options.max_depth.unwrap_or(std::usize::MAX));

        // 使用 rayon 进行并行处理
        walker
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| {
                !self.options.ignore_hidden || !entry.file_name().to_string_lossy().starts_with('.')
            })
            .par_bridge()
            .filter(|entry| filter.matches(entry))
            .map(|entry| entry.path().to_owned())
            .collect()
    }

    /// 统计目录中的子目录数量
    fn count_directories(&self, root: &PathBuf) -> usize {
        WalkDir::new(root)
            .follow_links(self.options.follow_links)
            .max_depth(self.options.max_depth.unwrap_or(std::usize::MAX))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_dir())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;
    use super::filter::NameFilter;

    #[test]
    fn test_finder_basic() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // 创建测试文件结构
        fs::create_dir(base_path.join("dir1")).unwrap();
        fs::create_dir(base_path.join("dir2")).unwrap();
        
        let mut file1 = File::create(base_path.join("dir1/test1.txt")).unwrap();
        file1.write_all(b"test content").unwrap();
        
        let mut file2 = File::create(base_path.join("dir2/test2.txt")).unwrap();
        file2.write_all(b"test content").unwrap();

        // 创建查找器
        let options = FindOptions::default();
        let finder = Finder::new(options);

        // 使用名称过滤器进行测试
        let filter = NameFilter::new("*.txt").unwrap();
        let results = finder.find(base_path.to_path_buf(), filter);

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|p| p.ends_with("test1.txt")));
        assert!(results.iter().any(|p| p.ends_with("test2.txt")));
    }

    #[test]
    fn test_finder_hidden_files() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // 创建测试文件结构，包括隐藏文件
        let mut hidden_file = File::create(base_path.join(".hidden.txt")).unwrap();
        hidden_file.write_all(b"hidden content").unwrap();
        
        let mut normal_file = File::create(base_path.join("normal.txt")).unwrap();
        normal_file.write_all(b"normal content").unwrap();

        // 测试不包含隐藏文件的情况
        let options = FindOptions::default();
        let finder = Finder::new(options);
        let filter = NameFilter::new("*.txt").unwrap();
        let results = finder.find(base_path.to_path_buf(), filter);
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("normal.txt"));

        // 测试包含隐藏文件的情况
        let mut options = FindOptions::default();
        options.ignore_hidden = false;
        let finder = Finder::new(options);
        let filter = NameFilter::new("*.txt").unwrap();
        let results = finder.find(base_path.to_path_buf(), filter);
        assert_eq!(results.len(), 2);
    }
}