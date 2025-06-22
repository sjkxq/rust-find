//! File filtering functionality
//!
//! This module provides filters for matching files based on various criteria.

use walkdir::DirEntry;
use glob::Pattern;

use crate::errors::{FindError, FindResult};

/// Trait for file filters
pub trait FileFilter {
    /// Check if the entry matches the filter
    fn matches(&self, entry: &DirEntry) -> bool;
    
    /// Get the filter description
    fn description(&self) -> String;
}

/// Factory for creating filters from command line arguments
pub struct FilterFactory;

impl FilterFactory {
    /// Create filters from command line arguments
    pub fn create_filters(
        name_patterns: Option<&[String]>,
        ignore_case: bool,
        absolute_path: bool,
        relative_path: bool,
    ) -> FindResult<Vec<Box<dyn FileFilter + Send + Sync>>> {
        let mut filters: Vec<Box<dyn FileFilter + Send + Sync>> = Vec::new();
        
        // Add name filters
        if let Some(patterns) = name_patterns {
            if !patterns.is_empty() {
                let name_filter = MultiNameFilter::new(patterns, ignore_case)?;
                filters.push(Box::new(name_filter));
            }
        }
        
        // Add path format filter if needed
        if absolute_path {
            filters.push(Box::new(PathFormatFilter::Absolute));
        } else if relative_path {
            filters.push(Box::new(PathFormatFilter::Relative));
        }
        
        Ok(filters)
    }
}

/// Filter for matching file names against a pattern
pub struct NameFilter {
    pattern: Pattern,
    original_pattern: String,
    ignore_case: bool,
}

impl NameFilter {
    /// Create a new NameFilter with the given pattern
    pub fn new(pattern: &str) -> FindResult<Self> {
        let compiled_pattern = Pattern::new(pattern)
            .map_err(|e| FindError::PatternError {
                message: format!("Invalid pattern '{}': {}", pattern, e),
            })?;
        
        Ok(Self {
            pattern: compiled_pattern,
            original_pattern: pattern.to_string(),
            ignore_case: false,
        })
    }
    
    /// Create a new case-insensitive NameFilter
    pub fn new_ignore_case(pattern: &str) -> FindResult<Self> {
        let mut filter = Self::new(pattern)?;
        filter.ignore_case = true;
        Ok(filter)
    }
}

impl FileFilter for NameFilter {
    fn matches(&self, entry: &DirEntry) -> bool {
        if let Some(name) = entry.file_name().to_str() {
            if self.ignore_case {
                // Case-insensitive matching
                let name_lower = name.to_lowercase();
                let pattern_lower = self.original_pattern.to_lowercase();
                return Pattern::new(&pattern_lower)
                    .map(|p| p.matches(&name_lower))
                    .unwrap_or(false);
            } else {
                // Case-sensitive matching
                return self.pattern.matches(name);
            }
        }
        false
    }
    
    fn description(&self) -> String {
        if self.ignore_case {
            format!("name (ignore case) matches '{}'", self.original_pattern)
        } else {
            format!("name matches '{}'", self.original_pattern)
        }
    }
}

/// Filter for matching file names against multiple patterns
pub struct MultiNameFilter {
    patterns: Vec<NameFilter>,
    any_match: bool,
}

impl MultiNameFilter {
    /// Create a new MultiNameFilter with the given patterns
    pub fn new(patterns: &[String], ignore_case: bool) -> FindResult<Self> {
        let mut name_filters = Vec::new();
        
        for pattern in patterns {
            let filter = if ignore_case {
                NameFilter::new_ignore_case(pattern)?
            } else {
                NameFilter::new(pattern)?
            };
            name_filters.push(filter);
        }
        
        Ok(Self {
            patterns: name_filters,
            any_match: true, // Default to OR logic
        })
    }
    
    /// Set whether any pattern match is sufficient (OR logic)
    /// or all patterns must match (AND logic)
    pub fn with_any_match(mut self, any_match: bool) -> Self {
        self.any_match = any_match;
        self
    }
}

impl FileFilter for MultiNameFilter {
    fn matches(&self, entry: &DirEntry) -> bool {
        if self.patterns.is_empty() {
            return true;
        }
        
        if self.any_match {
            // OR logic - any pattern can match
            self.patterns.iter().any(|filter| filter.matches(entry))
        } else {
            // AND logic - all patterns must match
            self.patterns.iter().all(|filter| filter.matches(entry))
        }
    }
    
    fn description(&self) -> String {
        let patterns: Vec<String> = self.patterns
            .iter()
            .map(|p| p.original_pattern.clone())
            .collect();
        
        let logic = if self.any_match { "any of" } else { "all of" };
        format!("name matches {} [{}]", logic, patterns.join(", "))
    }
}

/// Filter for matching file types
pub struct TypeFilter {
    file_type: FileType,
}

/// Supported file types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Symbolic link
    SymbolicLink,
}

impl TypeFilter {
    /// Create a new TypeFilter with the given type code
    pub fn new(type_code: &str) -> Result<Self, FindError> {
        let file_type = match type_code {
            "f" => FileType::File,
            "d" => FileType::Directory,
            "l" => FileType::SymbolicLink,
            _ => return Err(FindError::InvalidFileType(type_code.to_string())),
        };
        
        Ok(Self { file_type })
    }
}

impl FileFilter for TypeFilter {
    fn matches(&self, entry: &DirEntry) -> bool {
        match self.file_type {
            FileType::File => entry.file_type().is_file(),
            FileType::Directory => entry.file_type().is_dir(),
            FileType::SymbolicLink => entry.file_type().is_symlink(),
        }
    }
    
    fn description(&self) -> String {
        match self.file_type {
            FileType::File => "is a regular file".to_string(),
            FileType::Directory => "is a directory".to_string(),
            FileType::SymbolicLink => "is a symbolic link".to_string(),
        }
    }
}

/// Filter for controlling path format (absolute or relative)
pub enum PathFormatFilter {
    /// Output absolute paths
    Absolute,
    /// Output relative paths
    Relative,
}

impl FileFilter for PathFormatFilter {
    fn matches(&self, _entry: &DirEntry) -> bool {
        // This filter doesn't exclude any entries,
        // it just affects how they're displayed
        true
    }
    
    fn description(&self) -> String {
        match self {
            PathFormatFilter::Absolute => "output absolute paths".to_string(),
            PathFormatFilter::Relative => "output relative paths".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    
    fn create_test_entry(name: &str) -> Result<(TempDir, DirEntry), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join(name);
        File::create(&file_path)?.write_all(b"test")?;
        
        let entry = walkdir::WalkDir::new(&file_path)
            .into_iter()
            .next()
            .unwrap()?;
        
        Ok((temp_dir, entry))
    }
    
    #[test]
    fn test_name_filter() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, entry) = create_test_entry("test.txt")?;
        
        let filter = NameFilter::new("*.txt")?;
        assert!(filter.matches(&entry));
        
        let filter = NameFilter::new("*.rs")?;
        assert!(!filter.matches(&entry));
        
        Ok(())
    }
    
    #[test]
    fn test_name_filter_case_insensitive() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, entry) = create_test_entry("Test.TXT")?;
        
        let filter = NameFilter::new("*.txt")?;
        assert!(!filter.matches(&entry));
        
        let filter = NameFilter::new_ignore_case("*.txt")?;
        assert!(filter.matches(&entry));
        
        Ok(())
    }
    
    #[test]
    fn test_multi_name_filter() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, entry) = create_test_entry("test.txt")?;
        
        let filter = MultiNameFilter::new(&["*.txt".to_string(), "*.rs".to_string()], false)?;
        assert!(filter.matches(&entry));
        
        let filter = MultiNameFilter::new(&["*.doc".to_string(), "*.rs".to_string()], false)?;
        assert!(!filter.matches(&entry));
        
        Ok(())
    }
    
    #[test]
    fn test_type_filter() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create a file
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path)?.write_all(b"test")?;
        let file_entry = walkdir::WalkDir::new(&file_path)
            .into_iter()
            .next()
            .unwrap()?;
        
        // Create a directory
        let dir_path = temp_dir.path().join("testdir");
        std::fs::create_dir(&dir_path)?;
        let dir_entry = walkdir::WalkDir::new(&dir_path)
            .into_iter()
            .next()
            .unwrap()?;
        
        let file_filter = TypeFilter::new("f")?;
        assert!(file_filter.matches(&file_entry));
        assert!(!file_filter.matches(&dir_entry));
        
        let dir_filter = TypeFilter::new("d")?;
        assert!(!dir_filter.matches(&file_entry));
        assert!(dir_filter.matches(&dir_entry));
        
        Ok(())
    }
}