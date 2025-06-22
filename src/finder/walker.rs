//! File system traversal functionality
//!
//! This module provides the functionality for traversing the file system
//! and collecting file entries.

use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use crate::errors::{FindError, FindResult};
use super::options::FindOptions;

/// Handles file system traversal with the given options
pub struct FileWalker<'a> {
    options: &'a FindOptions,
}

impl<'a> FileWalker<'a> {
    /// Create a new FileWalker with the given options
    pub fn new(options: &'a FindOptions) -> Self {
        Self { options }
    }
    
    /// Walk the file system starting from the given path
    pub fn walk<P: AsRef<Path>>(&self, path: P) -> FindResult<Vec<DirEntry>> {
        let mut walker = WalkDir::new(path)
            .follow_links(self.options.follow_links);
        
        // Apply max depth if specified
        if let Some(depth) = self.options.max_depth {
            walker = walker.max_depth(depth);
        }
        
        let mut entries = Vec::new();
        let mut is_first = true;
        
        for entry in walker {
            match entry {
                Ok(entry) => {
                    if !is_first {
                        entries.push(entry);
                    }
                    is_first = false;
                }
                Err(err) => {
                    // Handle different error types based on options
                    if let Some(path) = err.path() {
                        match err.io_error() {
                            Some(io_err) => {
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
                            None => {
                                // Other walkdir errors
                                return Err(FindError::WalkDirError(err.to_string()));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(entries)
    }
}

/// Iterator-based file system walker
pub struct FileWalkerIterator<'a> {
    inner: walkdir::IntoIter,
    options: &'a FindOptions,
    skip_root: bool,
    root_path: std::path::PathBuf,
}

impl<'a> FileWalkerIterator<'a> {
    /// Create a new FileWalkerIterator with the given path and options
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
                    // Skip the root directory entry
                    if !self.skip_root {
                        self.skip_root = true;
                        if entry.path() == self.root_path {
                            continue;
                        }
                    }
                    return Some(Ok(entry));
                },
                Err(err) => {
                    // Handle errors based on options
                    if let Some(path) = err.path() {
                        if let Some(io_err) = err.io_error() {
                            match io_err.kind() {
                                std::io::ErrorKind::PermissionDenied => {
                                    if self.options.ignore_permission_errors {
                                        continue;
                                    }
                                    return Some(Err(FindError::PermissionDenied(path.to_path_buf())));
                                }
                                _ => {
                                    if self.options.ignore_io_errors {
                                        eprintln!("Warning: {}", err);
                                        continue;
                                    }
                                    return Some(Err(FindError::FilesystemError {
                                        source: std::io::Error::new(io_err.kind(), io_err.to_string()),
                                        path: path.to_path_buf(),
                                    }));
                                }
                            }
                        }
                    }
                    if self.options.ignore_io_errors {
                        eprintln!("Warning: {}", err);
                        continue;
                    }
                    return Some(Err(FindError::WalkDirError(err.to_string())));
                }
            }
        }
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