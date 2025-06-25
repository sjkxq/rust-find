use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir;

/// Result type for operations that can produce FindError
pub type FindResult<T> = Result<T, FindError>;

/// rust-find 的自定义错误类型
#[derive(Debug)]
pub enum FindError {
    /// 文件未找到
    FileNotFound(PathBuf),
    
    /// 权限不足
    PermissionDenied(PathBuf),
    
    /// 目录不可读
    DirectoryUnreadable(PathBuf),
    
    /// 符号链接问题
    SymlinkIssue(PathBuf),
    
    /// 文件系统错误（其他IO错误）
    FilesystemError {
        source: std::io::Error,
        path: PathBuf,
    },
    
    /// 指定的路径无效
    InvalidPath(PathBuf),
    
    /// 其他通用错误
    Other {
        message: String,
        context: Option<String>,
        timestamp: SystemTime,
    },

    /// 模式匹配错误
    PatternError {
        message: String,
    },

    /// 无效的文件类型
    InvalidFileType(String),

    /// 遍历目录时的错误
    WalkDirError(String),
}

impl fmt::Display for FindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FindError::FileNotFound(path) => 
                write!(f, "文件未找到: {}", path.display()),
            FindError::PermissionDenied(path) => 
                write!(f, "权限不足: {}", path.display()),
            FindError::DirectoryUnreadable(path) => 
                write!(f, "目录不可读: {}", path.display()),
            FindError::SymlinkIssue(path) => 
                write!(f, "符号链接问题: {}", path.display()),
            FindError::FilesystemError { source, path } => 
                write!(f, "文件系统错误 {}: {}", path.display(), source),
            FindError::InvalidPath(path) => 
                write!(f, "无效路径: {}", path.display()),
            FindError::Other { message, context, .. } => {
                write!(f, "错误: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, " (上下文: {})", ctx)?;
                }
                Ok(())
            },
            FindError::PatternError { message } => 
                write!(f, "模式匹配错误: {}", message),
            FindError::InvalidFileType(type_code) => 
                write!(f, "无效的文件类型: {}", type_code),
            FindError::WalkDirError(message) => 
                write!(f, "目录遍历错误: {}", message)
        }
    }
}

impl std::error::Error for FindError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FindError::FilesystemError { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FindError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => 
                FindError::FileNotFound(PathBuf::new()),
            std::io::ErrorKind::PermissionDenied => 
                FindError::PermissionDenied(PathBuf::new()),
            _ => FindError::FilesystemError {
                source: err,
                path: PathBuf::new(),
            },
        }
    }
}

impl From<walkdir::Error> for FindError {
    fn from(err: walkdir::Error) -> Self {
        let path = err.path().map(|p| p.to_path_buf()).unwrap_or_default();
        match err.io_error() {
            Some(io_err) => match io_err.kind() {
                std::io::ErrorKind::NotFound => FindError::FileNotFound(path),
                std::io::ErrorKind::PermissionDenied => FindError::PermissionDenied(path),
                _ => FindError::FilesystemError {
                    source: std::io::Error::new(io_err.kind(), io_err.to_string()),
                    path
                },
            },
            None => FindError::WalkDirError(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_filesystem_error_display() {
        // 测试文件系统错误的显示格式
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let find_error = FindError::FilesystemError {
            source: io_error,
            path: PathBuf::from("/test/path")
        };
        assert_eq!(
            find_error.to_string(),
            "文件系统错误 /test/path: file not found"
        );
    }

    #[test]
    fn test_invalid_path_display() {
        // 测试无效路径错误的显示格式
        let find_error = FindError::InvalidPath(PathBuf::from("/invalid/path"));
        assert_eq!(find_error.to_string(), "无效路径: /invalid/path");
    }

    #[test]
    fn test_other_error_display() {
        let find_error = FindError::Other {
            message: "出现了问题".to_string(),
            context: None,
            timestamp: SystemTime::now(),
        };
        assert_eq!(find_error.to_string(), "错误: 出现了问题");
    }

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::Other, "test error");
        let find_error: FindError = io_error.into();
        match find_error {
            FindError::FilesystemError { source: _, path } => assert_eq!(path, PathBuf::new()),
            _ => panic!("Expected FilesystemError variant"),
        }
    }
}