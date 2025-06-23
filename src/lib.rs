//! 用于查找文件和目录的库
//!
//! 本库提供了高性能的文件查找功能，支持：
//! - 多线程目录遍历
//! - 多种过滤条件（名称、类型、大小、时间等）
//! - 可定制的搜索选项
//! - 详细的错误报告
//!
//! ## 使用场景
//!
//! - 在项目中查找特定类型的文件
//! - 清理过时或大文件
//! - 构建自动化工具链
//!
//! # 示例
//!
//! 基本用法：
//! ```no_run
//! use rust_find::finder::{Finder, options::FindOptions};
//! use rust_find::finder::filter::{NameFilter, TypeFilter, FileFilter};
//!
//! // 创建查找器并设置选项
//! let options = FindOptions::new()
//!     .with_max_depth(Some(3))  // 最大搜索深度
//!     .with_follow_links(false); // 不跟随符号链接
//!
//! let mut finder = Finder::new(options);
//!
//! // 添加名称过滤器
//! let name_filter = NameFilter::new("*.rs").unwrap();
//! finder = finder.with_filter(Box::new(name_filter));
//!
//! // 执行查找
//! let results = finder.find(".").unwrap();
//!
//! // 输出结果
//! for entry in results {
//!     println!("找到文件: {}", entry.path().display());
//! }
//! ```
//!
//! 更多用法请参考各模块文档。

pub mod cli;
pub mod errors;
pub mod finder;

// Re-export main types for convenience
pub use errors::{FindError, FindResult};
pub use finder::Finder;