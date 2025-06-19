use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir;

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
    FilesystemError(std::io::Error, PathBuf),
    
    /// 指定的路径无效
    InvalidPath(PathBuf),
    
    /// 其他通用错误
    Other {
        message: String,
        context: Option<String>,
        timestamp: SystemTime,
    },
}

impl fmt::Display for FindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FindError::FileNotFound(path) => 
                write!(f, "File not found: {}", path.display()),
            FindError::PermissionDenied(path) => 
                write!(f, "Permission denied for: {}", path.display()),
            FindError::DirectoryUnreadable(path) => 
                write!(f, "Cannot read directory: {}", path.display()),
            FindError::SymlinkIssue(path) => 
                write!(f, "Symbolic link issue with: {}", path.display()),
            FindError::FilesystemError(e, path) => 
                write!(f, "Filesystem error at {}: {}", path.display(), e),
            FindError::InvalidPath(path) => 
                write!(f, "Invalid path: {}", path.display()),
            FindError::Other { message, context, .. } => {
                write!(f, "Error: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, " (context: {})", ctx)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for FindError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FindError::FilesystemError(e, _) => Some(e),
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
            _ => FindError::FilesystemError(err, PathBuf::new()),
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
                _ => FindError::FilesystemError(
                    std::io::Error::new(io_err.kind(), io_err.to_string()),
                    path
                ),
            },
            None => FindError::Other {
                message: err.to_string(),
                context: Some("walkdir error".to_string()),
                timestamp: SystemTime::now(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_filesystem_error_display() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let find_error = FindError::FilesystemError(io_error, PathBuf::from("/test/path"));
        assert_eq!(
            find_error.to_string(),
            "Filesystem error at /test/path: file not found"
        );
    }

    #[test]
    fn test_invalid_path_display() {
        let find_error = FindError::InvalidPath(PathBuf::from("/invalid/path"));
        assert_eq!(find_error.to_string(), "Invalid path: /invalid/path");
    }

    #[test]
    fn test_other_error_display() {
        let find_error = FindError::Other {
            message: "something went wrong".to_string(),
            context: None,
            timestamp: SystemTime::now(),
        };
        assert_eq!(find_error.to_string(), "Error: something went wrong");
    }

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::Other, "test error");
        let find_error: FindError = io_error.into();
        match find_error {
            FindError::FilesystemError(_, path) => assert_eq!(path, PathBuf::new()),
            _ => panic!("Expected FilesystemError variant"),
        }
    }
}