//! 文件系统遍历功能
//!
//! 本模块提供遍历文件系统并收集文件条目的功能。

use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use crate::errors::{FindError, FindResult};
use super::options::FindOptions;

/// 使用给定选项处理文件系统遍历
pub struct FileWalker<'a> {
    options: &'a FindOptions,
}

impl<'a> FileWalker<'a> {
    /// 使用给定选项创建新的 FileWalker
    pub fn new(options: &'a FindOptions) -> Self {
        Self { options }
    }
    
    /// 从给定路径开始遍历文件系统
    pub fn walk<P: AsRef<Path>>(&self, path: P) -> FindResult<Vec<DirEntry>> {
        let walker = self.init_walker(path.as_ref());
        let mut entries = Vec::new();
        let mut is_first = true;
        
        for entry in walker {
            let entry = self.process_entry(entry, &mut is_first)?;
            if let Some(entry) = entry {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }

    /// 使用配置的选项初始化目录遍历器
    fn init_walker(&self, path: &Path) -> walkdir::WalkDir {
        let mut walker = WalkDir::new(path)
            .follow_links(self.options.follow_links);
        
        if let Some(depth) = self.options.max_depth {
            walker = walker.max_depth(depth);
        }
        
        walker
    }

    /// 处理单个目录条目
    fn process_entry(
        &self,
        entry: Result<DirEntry, walkdir::Error>,
        is_first: &mut bool,
    ) -> FindResult<Option<DirEntry>> {
        match entry {
            Ok(entry) => {
                let should_include = !*is_first;
                *is_first = false;
                Ok(if should_include { Some(entry) } else { None })
            }
            Err(err) => self.handle_walk_error(err),
        }
    }

    /// 根据选项处理目录遍历错误
    fn handle_walk_error(&self, err: walkdir::Error) -> FindResult<Option<DirEntry>> {
        if let Some(path) = err.path() {
            if let Some(io_err) = err.io_error() {
                match io_err.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        if !self.options.ignore_permission_errors {
                            return Err(FindError::PermissionDenied(path.to_path_buf()));
                        }
                    }
                    _ => {
                        if !self.options.ignore_io_errors {
                            return Err(FindError::FilesystemError {
                                source: std::io::Error::new(io_err.kind(), io_err.to_string()),
                                path: path.to_path_buf(),
                            });
                        }
                    }
                }
            }
        }
        
        if !self.options.ignore_io_errors {
            return Err(FindError::WalkDirError(err.to_string()));
        }
        
        Ok(None)
    }
}

/// 基于迭代器的文件系统遍历器
pub struct FileWalkerIterator<'a> {
    inner: walkdir::IntoIter,
    options: &'a FindOptions,
    skip_root: bool,
    root_path: std::path::PathBuf,
}

impl<'a> FileWalkerIterator<'a> {
    /// 使用给定路径和选项创建新的 FileWalkerIterator
    pub fn new<P: AsRef<Path>>(path: P, options: &'a FindOptions) -> Self {
        let root_path = path.as_ref().to_path_buf();
        let mut walker = WalkDir::new(&root_path)
            .follow_links(options.follow_links);
        
        if let Some(depth) = options.max_depth {
            walker = walker.max_depth(depth);
        }
        
        Self {
            inner: walker.into_iter(),
            options,
            skip_root: false,
            root_path,
        }
    }
}

impl<'a> Iterator for FileWalkerIterator<'a> {
    type Item = FindResult<DirEntry>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok(entry) => {
                    if let Some(entry) = self.process_entry(entry) {
                        return Some(Ok(entry));
                    }
                },
                Err(err) => {
                    if let Some(result) = self.handle_error(err) {
                        return Some(result);
                    }
                }
            }
        }
    }
}

impl<'a> FileWalkerIterator<'a> {
    /// 处理目录条目，根据需要跳过根目录
    fn process_entry(&mut self, entry: DirEntry) -> Option<DirEntry> {
        if !self.skip_root {
            self.skip_root = true;
            if entry.path() == self.root_path {
                return None;
            }
        }
        Some(entry)
    }

    /// 根据选项处理目录遍历错误
    fn handle_error(&self, err: walkdir::Error) -> Option<FindResult<DirEntry>> {
        if let Some(path) = err.path() {
            if let Some(io_err) = err.io_error() {
                match io_err.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        if !self.options.ignore_permission_errors {
                            return Some(Err(FindError::PermissionDenied(path.to_path_buf())));
                        }
                    }
                    _ => {
                        if !self.options.ignore_io_errors {
                            return Some(Err(FindError::FilesystemError {
                                source: std::io::Error::new(io_err.kind(), io_err.to_string()),
                                path: path.to_path_buf(),
                            }));
                        }
                    }
                }
            }
        }

        if !self.options.ignore_io_errors {
            return Some(Err(FindError::WalkDirError(err.to_string())));
        }

        eprintln!("Warning: {}", err);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    
    fn create_test_structure() -> std::io::Result<TempDir> {
        let temp_dir = TempDir::new()?;
        
        // Create some files and directories
        File::create(temp_dir.path().join("file1.txt"))?.write_all(b"test")?;
        std::fs::create_dir(temp_dir.path().join("dir1"))?;
        File::create(temp_dir.path().join("dir1").join("file2.txt"))?.write_all(b"test")?;
        
        Ok(temp_dir)
    }
    
    #[test]
    fn test_file_walker() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_structure()?;
        let options = FindOptions::new();
        let walker = FileWalker::new(&options);
        
        let entries = walker.walk(temp_dir.path())?;
        
        // Should find 2 files + 1 subdir = 3 entries (root dir is skipped)
        assert_eq!(entries.len(), 3);
        
        Ok(())
    }
    
    #[test]
    fn test_file_walker_max_depth() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_structure()?;
        let options = FindOptions::new().with_max_depth(Some(1));
        let walker = FileWalker::new(&options);
        
        let entries = walker.walk(temp_dir.path())?;
        
        // Should find 1 file + 1 subdir = 2 entries (root dir is skipped)
        assert_eq!(entries.len(), 2);
        
        Ok(())
    }
    
    #[test]
    fn test_file_walker_iterator() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_structure()?;
        let options = FindOptions::new();
        let walker = FileWalkerIterator::new(temp_dir.path(), &options);
        
        let entries: Vec<_> = walker.collect::<Result<_, _>>()?;
        
        // Should find 2 files + 1 subdir = 3 entries (root dir is skipped)
        assert_eq!(entries.len(), 3);
        
        Ok(())
    }
}