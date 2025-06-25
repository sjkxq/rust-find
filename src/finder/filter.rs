//! 文件过滤功能模块
//!
//! 提供基于多种条件匹配文件的过滤器，包括：
//! - 文件名模式匹配
//! - 文件类型过滤
//! - 路径格式控制

use walkdir::DirEntry;
use glob::Pattern;

use crate::errors::{FindError, FindResult};

/// 文件过滤器trait
///
/// 定义所有文件过滤器必须实现的方法
pub trait FileFilter {
    /// 检查条目是否匹配过滤器
    ///
    /// # 参数
    /// - `entry`: 待检查的目录条目
    fn matches(&self, entry: &DirEntry) -> bool;
    
    /// 获取过滤器描述
    ///
    /// 用于生成用户友好的过滤器描述信息
    fn description(&self) -> String;
}

/// 过滤器工厂，用于从命令行参数创建过滤器
pub struct FilterFactory;

impl FilterFactory {
    /// 根据命令行参数创建过滤器集合
    ///
    /// # 参数
    /// - `name_patterns`: 文件名模式列表
    /// - `ignore_case`: 是否忽略大小写
    /// - `absolute_path`: 是否输出绝对路径
    /// - `relative_path`: 是否输出相对路径
    ///
    /// # 返回值
    /// 返回配置好的过滤器集合
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

/// 文件名模式过滤器
///
/// 根据文件名模式(支持glob语法)过滤文件
///
/// # 示例
/// ```
/// use rust_find::finder::filter::NameFilter;
///
/// // 创建大小写敏感的过滤器
/// let filter = NameFilter::new("*.rs").unwrap();
/// ```
pub struct NameFilter {
    pattern: Pattern,
    original_pattern: String,
    ignore_case: bool,
}

impl NameFilter {
    /// 创建新的文件名过滤器(大小写敏感)
    ///
    /// # 参数
    /// - `pattern`: 文件名模式(支持glob语法)
    ///
    /// # 错误
    /// 如果模式无效，返回PatternError错误
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
    
    /// 创建新的文件名过滤器(忽略大小写)
    ///
    /// # 参数
    /// - `pattern`: 文件名模式(支持glob语法)
    ///
    /// # 示例
    /// ```
    /// use rust_find::finder::filter::NameFilter;
    ///
    /// // 创建忽略大小写的过滤器
    /// let filter = NameFilter::new_ignore_case("*.RS").unwrap();
    /// ```
    pub fn new_ignore_case(pattern: &str) -> FindResult<Self> {
        let mut filter = Self::new(pattern)?;
        filter.ignore_case = true;
        Ok(filter)
    }
}

impl NameFilter {
    /// 执行大小写敏感匹配
    fn matches_case_sensitive(&self, name: &str) -> bool {
        self.pattern.matches(name)
    }

    /// 执行大小写不敏感匹配
    fn matches_case_insensitive(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        let pattern_lower = self.original_pattern.to_lowercase();
        Pattern::new(&pattern_lower)
            .map(|p| p.matches(&name_lower))
            .unwrap_or(false)
    }
}

impl FileFilter for NameFilter {
    fn matches(&self, entry: &DirEntry) -> bool {
        if let Some(name) = entry.file_name().to_str() {
            if self.ignore_case {
                self.matches_case_insensitive(name)
            } else {
                self.matches_case_sensitive(name)
            }
        } else {
            false
        }
    }
    
    fn description(&self) -> String {
        if self.ignore_case {
            format!("name (ignore case) matches '{}'", self.original_pattern)
        } else {
            format!("name matches '{}'", self.original_pattern)
        }
    }
}

/// 多模式文件名过滤器
///
/// 支持同时匹配多个文件名模式，可以使用AND或OR逻辑
///
/// # 示例
/// ```
/// use rust_find::finder::filter::MultiNameFilter;
///
/// // 创建匹配多个模式的过滤器
/// let filter = MultiNameFilter::new(&["*.rs".to_string(), "*.txt".to_string()], false).unwrap();
/// ```
pub struct MultiNameFilter {
    patterns: Vec<NameFilter>,
    any_match: bool,
}

impl MultiNameFilter {
    /// 创建新的多模式文件名过滤器
    ///
    /// # 参数
    /// - `patterns`: 文件名模式列表
    /// - `ignore_case`: 是否忽略大小写
    ///
    /// # 错误
    /// 如果任何模式无效，返回PatternError错误
    pub fn new(patterns: &[String], ignore_case: bool) -> FindResult<Self> {
        Self::validate_patterns(patterns)?;
        let patterns = Self::create_filters(patterns, ignore_case)?;
        
        Ok(Self {
            patterns,
            any_match: true, // Default to OR logic
        })
    }

    /// 在创建过滤器之前验证所有模式
    fn validate_patterns(patterns: &[String]) -> FindResult<()> {
        for pattern in patterns {
            if pattern.is_empty() {
                return Err(FindError::PatternError {
                    message: "Empty pattern is not allowed".to_string(),
                });
            }
        }
        Ok(())
    }

    /// 为每个模式创建单独的文件名过滤器
    fn create_filters(patterns: &[String], ignore_case: bool) -> FindResult<Vec<NameFilter>> {
        patterns
            .iter()
            .map(|pattern| {
                if ignore_case {
                    NameFilter::new_ignore_case(pattern)
                } else {
                    NameFilter::new(pattern)
                }
            })
            .collect()
    }
    
    /// 设置模式匹配逻辑
    ///
    /// # 参数
    /// - `any_match`: 
    ///   - true: 任一模式匹配即可（OR逻辑）
    ///   - false: 所有模式都必须匹配（AND逻辑）
    ///
    /// # 示例
    /// ```
    /// use rust_find::finder::filter::MultiNameFilter;
    ///
    /// // 创建OR逻辑的多模式过滤器
    /// let filter = MultiNameFilter::new(&["*.rs".to_string(), "*.txt".to_string()], false)
    ///     .unwrap()
    ///     .with_any_match(true);
    /// ```
    pub fn with_any_match(mut self, any_match: bool) -> Self {
        self.any_match = any_match;
        self
    }
}

impl FileFilter for MultiNameFilter {
    /// 检查文件是否匹配任一/所有模式
    fn matches(&self, entry: &DirEntry) -> bool {
        if self.patterns.is_empty() {
            return true;
        }
        
        if self.any_match {
            // OR逻辑 - 任一模式匹配即可
            self.patterns.iter().any(|filter| filter.matches(entry))
        } else {
            // AND逻辑 - 所有模式都必须匹配
            self.patterns.iter().all(|filter| filter.matches(entry))
        }
    }
    
    /// 获取过滤器的描述信息
    fn description(&self) -> String {
        let patterns: Vec<String> = self.patterns
            .iter()
            .map(|p| p.original_pattern.clone())
            .collect();
        
        let logic = if self.any_match { "任一" } else { "所有" };
        format!("文件名匹配{}模式 [{}]", logic, patterns.join(", "))
    }
}

/// 文件类型过滤器
///
/// 用于根据文件类型（普通文件、目录、符号链接）过滤文件
pub struct TypeFilter {
    file_type: FileType,
}

/// 支持的文件类型
///
/// 定义了系统支持的基本文件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    /// 普通文件
    File,
    /// 目录
    Directory,
    /// 符号链接
    SymbolicLink,
}

impl TypeFilter {
    /// 创建新的文件类型过滤器
    ///
    /// # 参数
    /// - `type_code`: 文件类型代码
    ///   - "f": 普通文件
    ///   - "d": 目录
    ///   - "l": 符号链接
    ///
    /// # 错误
    /// 如果类型代码无效，返回InvalidFileType错误
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

/// 控制路径格式（绝对或相对）的过滤器
pub enum PathFormatFilter {
    /// 输出绝对路径
    Absolute,
    /// 输出相对路径
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