//! find 工具的命令行接口
//!
//! 本模块提供了find工具的命令行接口，
//! 包括参数解析和验证功能。

use clap::Parser;
use crate::errors::FindError;
use crate::finder::options::FindOptions;

/// Linux find 命令的 Rust 实现
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 搜索路径（默认：当前目录）
    #[arg(default_value = ".")]
    pub paths: Vec<String>,

    /// 最大搜索深度
    #[arg(long, value_name = "NUM")]
    pub max_depth: Option<usize>,

    /// 跟随符号链接
    #[arg(short = 'L', long)]
    pub follow_links: bool,

    /// 启用调试日志
    #[arg(short, long)]
    pub debug: bool,

    /// 输出绝对路径
    #[arg(long)]
    pub absolute: bool,

    /// 输出相对路径（相对于当前目录）
    #[arg(long, conflicts_with = "absolute")]
    pub relative: bool,

    /// 按文件名模式匹配 (支持通配符，可多次指定)
    #[arg(short = 'n', long, conflicts_with = "iname")]
    pub name: Vec<String>,

    /// 不区分大小写的文件名匹配 (支持通配符，可多次指定)
    #[arg(short = 'i', long = "iname", conflicts_with = "name")]
    pub iname: Vec<String>,

    /// 并行搜索（实验性功能）
    #[arg(short = 'p', long)]
    pub parallel: bool,

    /// 忽略IO错误（如权限不足、符号链接循环等）
    #[arg(long)]
    pub ignore_io_errors: bool,

    /// 忽略权限错误
    #[arg(long)]
    pub ignore_permission_errors: bool,
}

impl Cli {
    /// 构建查找选项
    pub fn build_options(&self) -> FindOptions {
        FindOptions {
            max_depth: self.max_depth,
            follow_links: self.follow_links,
            ignore_permission_errors: self.ignore_permission_errors,
            ignore_io_errors: self.ignore_io_errors,
        }
    }

    /// 验证命令行参数
    pub fn validate(&self) -> Result<(), FindError> {
        // 验证路径
        for path in &self.paths {
            if !std::path::Path::new(path).exists() {
                return Err(FindError::FileNotFound(std::path::PathBuf::from(path)));
            }
        }

        // 验证最大深度
        if let Some(depth) = self.max_depth {
            if depth == 0 {
                return Err(FindError::Other {
                    message: "Maximum depth must be greater than 0".to_string(),
                    context: None,
                    timestamp: std::time::SystemTime::now(),
                });
            }
        }

        // 验证名称模式
        let patterns = if !self.name.is_empty() {
            &self.name
        } else {
            &self.iname
        };

        for pattern in patterns {
            if let Err(e) = glob::Pattern::new(pattern) {
                return Err(FindError::PatternError {
                    message: format!("Invalid pattern '{}': {}", pattern, e),
                });
            }
        }

        Ok(())
    }

    /// 检查是否忽略大小写
    pub fn ignore_case(&self) -> bool {
        !self.iname.is_empty()
    }

    /// 获取名称模式
    pub fn name_patterns(&self) -> &[String] {
        if !self.name.is_empty() {
            &self.name
        } else if !self.iname.is_empty() {
            &self.iname
        } else {
            &[]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_validation() {
        let cli = Cli {
            paths: vec![".".to_string()],
            max_depth: Some(1),
            follow_links: false,
            debug: false,
            absolute: false,
            relative: false,
            name: vec!["*.rs".to_string()],
            iname: vec![],
            parallel: false,
            ignore_io_errors: false,
            ignore_permission_errors: false,
        };

        assert!(cli.validate().is_ok());
    }

    #[test]
    fn test_cli_invalid_path() {
        let cli = Cli {
            paths: vec!["non_existent_path".to_string()],
            max_depth: Some(1),
            follow_links: false,
            debug: false,
            absolute: false,
            relative: false,
            name: vec![],
            iname: vec![],
            parallel: false,
            ignore_io_errors: false,
            ignore_permission_errors: false,
        };

        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_cli_invalid_pattern() {
        let cli = Cli {
            paths: vec![".".to_string()],
            max_depth: Some(1),
            follow_links: false,
            debug: false,
            absolute: false,
            relative: false,
            name: vec!["[".to_string()], // Invalid glob pattern
            iname: vec![],
            parallel: false,
            ignore_io_errors: false,
            ignore_permission_errors: false,
        };

        assert!(cli.validate().is_err());
    }
}