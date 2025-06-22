//! 用于查找文件和目录的库
//!
//! 本库提供了基于多种条件查找文件和目录的功能，
//! 包括名称模式、文件类型和搜索深度等条件。
//!
//! # Examples
//!
//! ```no_run
//! use rust_find::finder::{Finder, options::FindOptions};
//! use rust_find::finder::filter::{NameFilter, TypeFilter, FileFilter};
//!
//! // Create a finder with options
//! let options = FindOptions::new()
//!     .with_max_depth(Some(3))
//!     .with_follow_links(false);
//!
//! let mut finder = Finder::new(options);
//!
//! // Add filters
//! let name_filter = NameFilter::new("*.rs").unwrap();
//! finder = finder.with_filter(Box::new(name_filter));
//!
//! // Find files
//! let results = finder.find(".").unwrap();
//!
//! for entry in results {
//!     println!("{}", entry.path().display());
//! }
//! ```

pub mod cli;
pub mod errors;
pub mod finder;

// Re-export main types for convenience
pub use errors::{FindError, FindResult};
pub use finder::Finder;