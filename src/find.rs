use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc::Sender};
use std::time::SystemTime;
use log::{debug, error, warn};
use rayon::iter::{ParallelBridge, ParallelIterator};
use crate::errors::FindError;

/// 文件搜索选项
#[derive(Debug, Default, Clone)]
pub struct FindOptions {
    /// 最大搜索深度（None表示无限制）
    pub max_depth: Option<usize>,
    /// 是否跟随符号链接
    pub follow_links: bool,
    /// 输出绝对路径
    pub absolute_path: bool,
    /// 输出相对路径（相对于当前工作目录）
    pub relative_path: bool,
    /// 是否使用并行遍历（默认true）
    pub parallel: bool,
    /// 文件名匹配模式列表（支持通配符）
    pub name_patterns: Vec<String>,
    /// 是否不区分大小写匹配
    pub ignore_case: bool,
}

impl FindOptions {
    /// 将路径转换为所需格式
    pub fn format_path(&self, path: &Path) -> PathBuf {
        if self.absolute_path {
            // 转换为绝对路径
            if let Ok(abs_path) = path.canonicalize() {
                return abs_path;
            }
        } else if self.relative_path {
            // 转换为相对路径
            if let Ok(rel_path) = path.strip_prefix(std::env::current_dir().unwrap_or_default()) {
                return rel_path.to_path_buf();
            }
        }
        // 默认返回原始路径
        path.to_path_buf()
    }
}

/// 查找符合给定条件的文件
pub fn find_files<P: AsRef<Path>>(
    path: P,
    options: &FindOptions,
) -> Result<Vec<PathBuf>, FindError> {
    let path = path.as_ref();
    debug!("Searching in path: {}", path.display());
    
    if !path.exists() {
        return Err(FindError::FileNotFound(path.to_path_buf()));
    }

    if !path.is_dir() {
        return Err(FindError::Other {
                                message: format!("路径不是一个目录: {}", path.display()),
            context: Some("搜索需要一个目录".to_string()),
            timestamp: SystemTime::now(),
        });
    }

    let mut results = if options.parallel {
        parallel_traverse_directory(path, Arc::new(options.clone()))?
    } else {
        let mut results = Vec::new();
        traverse_directory(path, 0, options, &mut results)?;
        results
    };

    // 如果指定了名称模式过滤条件，则应用过滤
    if !options.name_patterns.is_empty() {
        results = results.into_iter().filter(|path| {
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    let text_to_match = if options.ignore_case {
                        file_name_str.to_lowercase()
                    } else {
                        file_name_str.to_string()
                    };

                    return options.name_patterns.iter().any(|pattern| {
                        let pattern_to_match = if options.ignore_case {
                            pattern.to_lowercase()
                        } else {
                            pattern.clone()
                        };
                        
                        glob::Pattern::new(&pattern_to_match)
                            .map(|p| p.matches(&text_to_match))
                            .unwrap_or(false)
                    });
                }
            }
            false
        }).collect();
    }

    Ok(results)
}

/// 并行目录遍历实现
fn parallel_traverse_directory(
    path: &Path,
    options: Arc<FindOptions>,
) -> Result<Vec<PathBuf>, FindError> {
    use std::sync::mpsc::channel;
    use std::sync::Arc;

    let (sender, receiver) = channel();
    let sender = Arc::new(sender);

    // Start parallel traversal
    rayon::scope(|s| {
        s.spawn(|_| {
            let sender_clone = sender.clone();
            if let Err(e) = parallel_traverse_impl(path, 0, options, sender) {
                let _ = sender_clone.send(Err(e));
            }
        });
    });

    // Collect results
    let mut results = Vec::new();
    for item in receiver {
        match item {
            Ok(path) => results.push(path),
            Err(e) => return Err(e),
        }
    }

    Ok(results)
}

/// 并行遍历实现
fn parallel_traverse_impl(
    path: &Path,
    current_depth: usize,
    options: Arc<FindOptions>,
    sender: Arc<Sender<Result<PathBuf, FindError>>>,
) -> Result<(), FindError> {
    // Check depth limit
    if let Some(max_depth) = options.max_depth {
        if current_depth > max_depth {
            return Ok(());
        }
    }

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            return Err(FindError::FilesystemError {
                source: e,
                path: path.to_path_buf()
            });
        }
    };

    // 并行处理目录条目
    entries.par_bridge().for_each_with(sender.clone(), |s, entry| {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                let _ = s.send(Err(FindError::FilesystemError {
                    source: e,
                    path: path.to_path_buf()
                }));
                return;
            }
        };

        let path = entry.path();
        let _ = s.send(Ok(options.format_path(&path)));

        // Handle directories
        if path.is_dir() {
            let is_link = is_symlink(&path);
            if !is_link || (is_link && options.follow_links) {
                if let Err(e) = parallel_traverse_impl(&path, current_depth + 1, options.clone(), s.clone()) {
                    let _ = s.send(Err(e));
                }
            }
        }
    });

    Ok(())
}

/// 递归遍历目录
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
        // At max depth, only process current directory
        if current_depth == max_depth {
            // Don't recurse but still process current directory
            let entries = match std::fs::read_dir(path) {
                Ok(entries) => entries,
                Err(e) => {
                    let error = match e.kind() {
                        std::io::ErrorKind::PermissionDenied => {
                            warn!("没有权限读取目录: {}", path.display());
                            FindError::PermissionDenied(path.to_path_buf())
                        }
                        std::io::ErrorKind::NotFound => {
                            warn!("目录未找到: {}", path.display());
                            FindError::FileNotFound(path.to_path_buf())
                        }
                        _ => {
                            error!("读取目录时出错 {}: {}", path.display(), e);
                            FindError::FilesystemError(e, path.to_path_buf())
                        }
                    };
                    return Err(error);
                }
            };

            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        match e.kind() {
                            std::io::ErrorKind::PermissionDenied => {
                                warn!("跳过条目(权限被拒绝)在 {}: {}", path.display(), e);
                                continue;
                            }
                            std::io::ErrorKind::NotFound => {
                                warn!("跳过 {} 中的缺失条目: {}", path.display(), e);
                                continue;
                            }
                            _ => {
                                error!("读取目录 {} 中的条目时出错: {}", path.display(), e);
                                continue;
                            }
                        };
                    }
                };
                results.push(options.format_path(&entry.path()));
            }
            return Ok(());
        }
    }

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Error reading directory {}: {}", path.display(), e);
            return Err(FindError::FilesystemError {
                source: e,
                path: path.to_path_buf()
            });
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
        results.push(options.format_path(&path));

        // Handle directories
        if path.is_dir() {
            if is_symlink(&path) {
                debug!("发现符号链接: {}", path.display());
                if options.follow_links {
                    debug!("正在跟随符号链接: {}", path.display());
                    if let Err(e) = traverse_directory(&path, current_depth + 1, options, results) {
                        error!("跟随符号链接时出错 {}: {}", path.display(), e);
                    }
                }
            } else {
                // Regular directory
                if let Err(e) = traverse_directory(&path, current_depth + 1, options, results) {
                                            error!("遍历目录时出错 {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}

/// 检查路径是否为符号链接
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

    #[test]
    fn test_parallel_vs_serial() {
        let dir = tempdir().unwrap();
        // Create test files
        File::create(dir.path().join("file1.txt")).unwrap();
        File::create(dir.path().join("file2.txt")).unwrap();
        // Create subdirectory
        let subdir = tempfile::tempdir_in(dir.path()).unwrap();
        File::create(subdir.path().join("file3.txt")).unwrap();

        // Test with parallel mode
        let parallel_options = FindOptions {
            parallel: true,
            ..Default::default()
        };
        let parallel_result = find_files(dir.path(), &parallel_options).unwrap();

        // Test with serial mode
        let serial_options = FindOptions {
            parallel: false,
            ..Default::default()
        };
        let serial_result = find_files(dir.path(), &serial_options).unwrap();

        // Results should be the same (order may differ)
        assert_eq!(parallel_result.len(), serial_result.len());
        assert!(parallel_result.iter().all(|p| serial_result.contains(p)));
        assert!(serial_result.iter().all(|p| parallel_result.contains(p)));
    }
}