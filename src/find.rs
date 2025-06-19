use std::path::{Path, PathBuf};
use log::{debug, error};
use crate::errors::FindError;

/// Options for file search
#[derive(Debug, Default)]
pub struct FindOptions {
    /// Maximum depth to search (None for unlimited)
    pub max_depth: Option<usize>,
    /// Follow symbolic links
    pub follow_links: bool,
}

/// Find files matching given criteria
pub fn find_files<P: AsRef<Path>>(
    path: P,
    options: &FindOptions,
) -> Result<Vec<PathBuf>, FindError> {
    let path = path.as_ref();
    debug!("Searching in path: {}", path.display());
    
    if !path.exists() {
        return Err(FindError::InvalidPath(path.to_path_buf()));
    }

    let mut results = Vec::new();
    traverse_directory(path, 0, options, &mut results)?;
    Ok(results)
}

/// Recursively traverse directory
fn traverse_directory(
    path: &Path,
    current_depth: usize,
    options: &FindOptions,
    results: &mut Vec<PathBuf>,
) -> Result<(), FindError> {
    // Check depth limit
    if let Some(max_depth) = options.max_depth {
        if current_depth > max_depth {
            return Ok(());
        }
        // Only include the directory itself when max_depth=0
        if current_depth == max_depth {
            results.push(path.to_path_buf());
            return Ok(());
        }
    }

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Error reading directory {}: {}", path.display(), e);
            return Err(FindError::IoError(e, path.to_path_buf()));
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("Error reading directory entry in {}: {}", path.display(), e);
                continue;
            }
        };

        let path = entry.path();
        results.push(path.clone());

        // Recursively traverse subdirectories
        if path.is_dir() && (!options.follow_links || !is_symlink(&path)) {
            if let Err(e) = traverse_directory(&path, current_depth + 1, options, results) {
                error!("Error traversing {}: {}", path.display(), e);
            }
        }
    }

    Ok(())
}

/// Check if path is a symbolic link
fn is_symlink<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_find_files_in_empty_dir() {
        let dir = tempdir().unwrap();
        let options = FindOptions::default();
        let result = find_files(dir.path(), &options).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_files_with_files() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("file1.txt")).unwrap();
        File::create(dir.path().join("file2.txt")).unwrap();
        
        let options = FindOptions::default();
        let result = find_files(dir.path(), &options).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_files_with_max_depth() {
        let dir = tempdir().unwrap();
        let subdir = tempfile::tempdir_in(dir.path()).unwrap();
        File::create(subdir.path().join("file.txt")).unwrap();
        
        let options = FindOptions {
            max_depth: Some(0),
            ..Default::default()
        };
        let result = find_files(dir.path(), &options).unwrap();
        assert_eq!(result.len(), 1); // Only the dir itself
    }

    #[test]
    fn test_is_symlink() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();
        
        #[cfg(unix)]
        std::os::unix::fs::symlink(&file_path, dir.path().join("symlink.txt")).unwrap();
        
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&file_path, dir.path().join("symlink.txt")).unwrap();
        
        assert!(!is_symlink(&file_path));
        assert!(is_symlink(dir.path().join("symlink.txt")));
    }
}