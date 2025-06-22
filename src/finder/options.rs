//! Options for file finding
//!
//! This module provides options for configuring the file finding process.

use crate::cli::Cli;

/// Options for configuring the file finding process
#[derive(Debug, Clone)]
pub struct FindOptions {
    /// Maximum depth to search
    pub max_depth: Option<usize>,
    
    /// Whether to follow symbolic links
    pub follow_links: bool,
    
    /// Whether to ignore permission errors
    pub ignore_permission_errors: bool,
    
    /// Whether to ignore I/O errors
    pub ignore_io_errors: bool,
}

impl FindOptions {
    /// Create a new FindOptions with default values
    pub fn new() -> Self {
        Self {
            max_depth: None,
            follow_links: false,
            ignore_permission_errors: true,
            ignore_io_errors: false,
        }
    }
    
    /// Set the maximum depth to search
    pub fn with_max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }
    
    /// Set whether to follow symbolic links
    pub fn with_follow_links(mut self, follow_links: bool) -> Self {
        self.follow_links = follow_links;
        self
    }
    
    /// Set whether to ignore permission errors
    pub fn with_ignore_permission_errors(mut self, ignore: bool) -> Self {
        self.ignore_permission_errors = ignore;
        self
    }
    
    /// Set whether to ignore I/O errors
    pub fn with_ignore_io_errors(mut self, ignore: bool) -> Self {
        self.ignore_io_errors = ignore;
        self
    }
    
    /// Create FindOptions from CLI arguments
    pub fn from_cli(cli: &Cli) -> Self {
        Self::new()
            .with_max_depth(cli.max_depth)
            .with_follow_links(cli.follow_links)
    }
}

impl Default for FindOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_find_options_defaults() {
        let options = FindOptions::new();
        assert_eq!(options.max_depth, None);
        assert_eq!(options.follow_links, false);
        assert_eq!(options.ignore_permission_errors, true);
        assert_eq!(options.ignore_io_errors, false);
    }
    
    #[test]
    fn test_find_options_with_max_depth() {
        let options = FindOptions::new().with_max_depth(Some(3));
        assert_eq!(options.max_depth, Some(3));
    }
    
    #[test]
    fn test_find_options_with_follow_links() {
        let options = FindOptions::new().with_follow_links(true);
        assert_eq!(options.follow_links, true);
    }
}