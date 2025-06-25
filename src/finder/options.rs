//! 文件查找配置选项
//!
//! 提供用于配置文件查找过程的选项，包括：
//! - 搜索深度控制
//! - 符号链接处理
//! - 错误处理策略

use crate::cli::Cli;

/// 文件查找配置选项
///
/// 用于配置文件查找过程的各种参数，支持链式调用配置。
///
/// # 示例
/// ```
/// use rust_find::finder::options::FindOptions;
///
/// let options = FindOptions::new()
///     .with_max_depth(Some(5))
///     .with_follow_links(true);
/// ```
#[derive(Debug, Clone)]
pub struct FindOptions {
    /// 最大搜索深度，None表示不限制
    pub max_depth: Option<usize>,
    
    /// 是否跟随符号链接，默认为false
    pub follow_links: bool,
    
    /// 是否忽略权限错误，默认为true
    pub ignore_permission_errors: bool,
    
    /// 是否忽略I/O错误，默认为false
    pub ignore_io_errors: bool,
    
    /// 是否忽略隐藏文件，默认为true
    pub ignore_hidden: bool,
    
    /// 线程池最大线程数，默认为CPU核心数
    pub max_threads: usize,
    
    /// 线程池最小线程数，默认为1
    pub min_threads: usize,
    
    /// 每个线程处理的目录数，默认为10
    pub dirs_per_thread: usize,
    
    /// 是否自动调整线程数，默认为true
    pub auto_adjust: bool,
}

impl FindOptions {
    /// 创建新的配置选项实例，使用默认值
    ///
    /// 默认值：
    /// - max_depth: None (不限制深度)
    /// - follow_links: false
    /// - ignore_permission_errors: true
    /// - ignore_io_errors: false
    pub fn new() -> Self {
        let num_cpus = num_cpus::get();
        Self {
            max_depth: None,
            follow_links: false,
            ignore_permission_errors: true,
            ignore_io_errors: false,
            ignore_hidden: true,
            max_threads: num_cpus,
            min_threads: 1,
            dirs_per_thread: 10,
            auto_adjust: true,
        }
    }
    
    /// 设置最大搜索深度
    ///
    /// # 参数
    /// - `max_depth`: 最大深度值，None表示不限制
    pub fn with_max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }
    
    /// 设置是否跟随符号链接
    ///
    /// # 参数
    /// - `follow_links`: true表示跟随符号链接
    pub fn with_follow_links(mut self, follow_links: bool) -> Self {
        self.follow_links = follow_links;
        self
    }
    
    /// 设置是否忽略权限错误
    ///
    /// # 参数
    /// - `ignore`: true表示忽略权限错误
    pub fn with_ignore_permission_errors(mut self, ignore: bool) -> Self {
        self.ignore_permission_errors = ignore;
        self
    }
    
    /// 设置是否忽略I/O错误
    ///
    /// # 参数
    /// - `ignore`: true表示忽略I/O错误
    pub fn with_ignore_io_errors(mut self, ignore: bool) -> Self {
        self.ignore_io_errors = ignore;
        self
    }
    
    /// 设置是否忽略隐藏文件
    ///
    /// # 参数
    /// - `ignore`: true表示忽略隐藏文件
    pub fn with_ignore_hidden(mut self, ignore: bool) -> Self {
        self.ignore_hidden = ignore;
        self
    }
    
    /// 设置线程池最大线程数
    ///
    /// # 参数
    /// - `max`: 最大线程数
    pub fn with_max_threads(mut self, max: usize) -> Self {
        self.max_threads = max;
        self
    }
    
    /// 设置线程池最小线程数
    ///
    /// # 参数
    /// - `min`: 最小线程数
    pub fn with_min_threads(mut self, min: usize) -> Self {
        self.min_threads = min;
        self
    }
    
    /// 设置每个线程处理的目录数
    ///
    /// # 参数
    /// - `count`: 每个线程处理的目录数
    pub fn with_dirs_per_thread(mut self, count: usize) -> Self {
        self.dirs_per_thread = count;
        self
    }
    
    /// 设置是否自动调整线程数
    ///
    /// # 参数
    /// - `adjust`: true表示自动调整线程数
    pub fn with_auto_adjust(mut self, adjust: bool) -> Self {
        self.auto_adjust = adjust;
        self
    }
    
    /// 从命令行参数创建配置选项
    ///
    /// # 参数
    /// - `cli`: 命令行参数解析结果
    pub fn from_cli(cli: &Cli) -> Self {
        Self::new()
            .with_max_depth(cli.max_depth)
            .with_follow_links(cli.follow_links)
            .with_ignore_permission_errors(cli.ignore_permission_errors)
            .with_ignore_io_errors(cli.ignore_io_errors)
            .with_ignore_hidden(!cli.no_ignore_hidden)
            .with_max_threads(cli.max_threads.unwrap_or(num_cpus::get()))
            .with_min_threads(cli.min_threads.unwrap_or(1))
            .with_dirs_per_thread(cli.dirs_per_thread.unwrap_or(10))
            .with_auto_adjust(!cli.no_auto_adjust)
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