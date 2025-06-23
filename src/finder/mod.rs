//! 文件查找功能模块
//!
//! 提供基于多种条件查找文件和目录的核心功能，包括：
//! - 单线程和并行查找
//! - 多种过滤条件组合
//! - 可定制的搜索选项

pub mod filter;
pub mod options;
pub mod walker;

use std::path::Path;
use rayon::prelude::*;
use walkdir::DirEntry;

use crate::errors::{FindResult};
use self::filter::FileFilter;
use self::options::FindOptions;
use self::walker::{FileWalker, FileWalkerIterator};

/// 文件查找器，负责协调整个搜索过程
///
/// # 示例
/// ```
/// use rust_find::finder::{Finder, options::FindOptions};
/// use rust_find::finder::filter::NameFilter;
///
/// let options = FindOptions::new().with_max_depth(Some(3));
/// let name_filter = NameFilter::new("*.rs").unwrap();
///
/// let finder = Finder::new(options).with_filter(Box::new(name_filter));
/// let results = finder.find(".").unwrap();
/// ```
pub struct Finder {
    options: FindOptions,
    filters: Vec<Box<dyn FileFilter + Send + Sync>>,
}

impl Finder {
    /// 创建新的查找器实例
    ///
    /// # 参数
    /// - `options`: 查找选项配置
    pub fn new(options: FindOptions) -> Self {
        Self {
            options,
            filters: Vec::new(),
        }
    }
    
    /// 添加过滤器到查找器
    ///
    /// # 参数
    /// - `filter`: 文件过滤器，需实现FileFilter trait
    ///
    /// # 返回值
    /// 返回修改后的Finder实例，支持链式调用
    pub fn with_filter(mut self, filter: Box<dyn FileFilter + Send + Sync>) -> Self {
        self.filters.push(filter);
        self
    }
    
    /// 查找匹配条件的文件和目录（单线程）
    ///
    /// # 参数
    /// - `path`: 搜索起始路径
    ///
    /// # 返回值
    /// 返回匹配的目录条目列表
    pub fn find<P: AsRef<Path>>(&self, path: P) -> FindResult<Vec<DirEntry>> {
        let walker = FileWalker::new(&self.options);
        let entries = walker.walk(path)?;
        
        // Apply filters
        let filtered_entries = entries
            .into_iter()
            .filter(|entry| self.matches_filters(entry))
            .collect();
        
        Ok(filtered_entries)
    }
    
    /// Find files and directories matching the criteria in parallel
    pub fn find_parallel<P: AsRef<Path>>(&self, path: P) -> FindResult<Vec<DirEntry>> {
        let walker = FileWalkerIterator::new(path, &self.options);
        
        // Collect entries, filtering out errors
        let entries: Vec<_> = walker
            .filter_map(|result| match result {
                Ok(entry) => Some(entry),
                Err(err) => {
                    log::warn!("Error walking directory: {}", err);
                    None
                }
            })
            .collect();
        
        // Apply filters in parallel
        let filtered_entries = entries
            .into_par_iter()
            .filter(|entry| self.matches_filters(entry))
            .collect();
        
        Ok(filtered_entries)
    }
    
    /// Check if an entry matches all filters
    fn matches_filters(&self, entry: &DirEntry) -> bool {
        self.filters.iter().all(|filter| filter.matches(entry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    use crate::finder::filter::{NameFilter, TypeFilter};
    
    fn create_test_file(dir: &Path, name: &str) -> std::io::Result<()> {
        let path = dir.join(name);
        let mut file = File::create(path)?;
        file.write_all(b"test content")?;
        Ok(())
    }
    
    #[test]
    fn test_finder_with_name_filter() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(temp_dir.path(), "test.txt")?;
        create_test_file(temp_dir.path(), "other.txt")?;
        
        let options = FindOptions::new();
        let name_filter = NameFilter::new("test.*")?;
        
        let finder = Finder::new(options).with_filter(Box::new(name_filter));
        let results = finder.find(temp_dir.path())?;
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name().to_str().unwrap(), "test.txt");
        
        Ok(())
    }
    
    #[test]
    fn test_finder_with_multiple_filters() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(temp_dir.path(), "test.txt")?;
        std::fs::create_dir(temp_dir.path().join("test_dir"))?;
        
        let options = FindOptions::new();
        let name_filter = NameFilter::new("test*")?;
        let type_filter = TypeFilter::new("f").unwrap();
        
        let finder = Finder::new(options)
            .with_filter(Box::new(name_filter))
            .with_filter(Box::new(type_filter));
        
        let results = finder.find(temp_dir.path())?;
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name().to_str().unwrap(), "test.txt");
        
        Ok(())
    }
}