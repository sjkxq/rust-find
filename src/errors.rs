use std::fmt;
use std::path::PathBuf;

/// Custom error types for rust-find
#[derive(Debug)]
pub enum FindError {
    /// IO related errors (permission denied, not found, etc.)
    IoError(std::io::Error, PathBuf),
    
    /// Invalid path specified
    InvalidPath(PathBuf),
    
    /// Other general errors
    Other(String),
}

impl fmt::Display for FindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FindError::IoError(e, path) => write!(f, "IO error at {}: {}", path.display(), e),
            FindError::InvalidPath(path) => write!(f, "Invalid path: {}", path.display()),
            FindError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for FindError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FindError::IoError(e, _) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FindError {
    fn from(err: std::io::Error) -> Self {
        FindError::IoError(err, PathBuf::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_display() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let find_error = FindError::IoError(io_error, PathBuf::from("/test/path"));
        assert_eq!(
            find_error.to_string(),
            "IO error at /test/path: file not found"
        );
    }

    #[test]
    fn test_invalid_path_display() {
        let find_error = FindError::InvalidPath(PathBuf::from("/invalid/path"));
        assert_eq!(find_error.to_string(), "Invalid path: /invalid/path");
    }

    #[test]
    fn test_other_error_display() {
        let find_error = FindError::Other("something went wrong".to_string());
        assert_eq!(find_error.to_string(), "Error: something went wrong");
    }

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::Other, "test error");
        let find_error: FindError = io_error.into();
        match find_error {
            FindError::IoError(_, path) => assert_eq!(path, PathBuf::new()),
            _ => panic!("Expected IoError variant"),
        }
    }
}