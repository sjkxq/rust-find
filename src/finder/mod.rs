//! File finding functionality
//!
//! This module provides the core functionality for finding files and directories
//! based on various criteria.

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

/// Main finder struct that coordinates the search process
pub struct Finder {
    options: FindOptions,
    filters: Vec<Box<dyn FileFilter + Send + Sync>>,
}

impl Finder {
    /// Create a new Finder with the given options
    pub fn new(options: FindOptions) -> Self {
        Self {
            options,
            filters: Vec::new(),
        }
    }
    
    /// Add a filter to the finder
    pub fn with_filter(mut self, filter: Box<dyn FileFilter + Send + Sync>) -> Self {
        self.filters.push(filter);
        self
    }
    
    /// Find files and directories matching the criteria
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