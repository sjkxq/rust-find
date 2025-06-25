//! 自适应线程池模块
//! 
//! 提供根据系统资源和工作负载自动调整线程数量的线程池实现。

use std::sync::atomic::{AtomicUsize, Ordering};
use log::{debug, info};
use num_cpus;

/// 线程池配置选项
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// 最小线程数
    pub min_threads: usize,
    /// 最大线程数
    pub max_threads: usize,
    /// 每个线程处理的目录数量
    pub dirs_per_thread: usize,
    /// 是否自动调整线程数
    pub auto_adjust: bool,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            min_threads: 2,
            max_threads: num_cpus::get(),
            dirs_per_thread: 100,
            auto_adjust: true,
        }
    }
}

/// 自适应线程池
/// 
/// 根据系统资源和工作负载自动调整线程数量的线程池实现。
#[derive(Debug)]
pub struct AdaptiveThreadPool {
    /// 线程池配置
    config: ThreadPoolConfig,
    /// 当前目录数量
    directory_count: AtomicUsize,
    /// 当前线程数量
    current_threads: AtomicUsize,
}

impl AdaptiveThreadPool {
    /// 创建新的自适应线程池
    pub fn new(config: ThreadPoolConfig) -> Self {
        let initial_threads = config.min_threads;
        
        Self {
            config,
            directory_count: AtomicUsize::new(0),
            current_threads: AtomicUsize::new(initial_threads),
        }
    }
    
    /// 更新目录数量
    pub fn update_directory_count(&self, count: usize) {
        self.directory_count.store(count, Ordering::Relaxed);
        debug!("Updated directory count to {}", count);
    }
    
    /// 调整线程数量并返回新的线程数
    pub fn adjust_thread_count(&self) -> usize {
        if !self.config.auto_adjust {
            let threads = self.current_threads.load(Ordering::Relaxed);
            debug!("Auto-adjust disabled, using {} threads", threads);
            return threads;
        }
        
        let dir_count = self.directory_count.load(Ordering::Relaxed);
        let cpu_count = num_cpus::get();
        
        debug!("Adjusting thread count - dirs: {}, min: {}, max: {}, per_thread: {}, cpus: {}",
              dir_count, self.config.min_threads, self.config.max_threads,
              self.config.dirs_per_thread, cpu_count);
        
        // 计算理想线程数
        let new_threads = if dir_count == 0 {
            self.config.min_threads
        } else {
            let ideal_threads = (dir_count as f64 / self.config.dirs_per_thread as f64).ceil() as usize;
            // 确保线程数在配置范围内
            ideal_threads
                .max(self.config.min_threads)  // 至少使用min_threads
                .min(self.config.max_threads)   // 不超过max_threads
                .min(cpu_count.max(self.config.min_threads)) // 不超过CPU核心数，但至少使用min_threads
        };
        
        // 更新并返回新的线程数
        self.current_threads.store(new_threads, Ordering::Relaxed);
        info!("Adjusted thread count to {} (directories: {}, CPUs: {})", 
              new_threads, dir_count, cpu_count);
        
        new_threads
    }
    
    /// 获取当前线程数
    pub fn get_thread_count(&self) -> usize {
        self.current_threads.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thread_pool_config_default() {
        let config = ThreadPoolConfig::default();
        assert_eq!(config.min_threads, 2);
        assert_eq!(config.max_threads, num_cpus::get());
        assert_eq!(config.dirs_per_thread, 100);
        assert!(config.auto_adjust);
    }
    
    #[test]
    fn test_adaptive_thread_pool_adjust() {
        // 测试少量目录的情况 (小于 dirs_per_thread)
        let config = ThreadPoolConfig {
            min_threads: 2,
            max_threads: 8,
            dirs_per_thread: 100,
            auto_adjust: true,
        };
        let pool = AdaptiveThreadPool::new(config.clone());
        pool.update_directory_count(50);
        let threads = pool.adjust_thread_count();
        assert!(threads >= config.min_threads && threads <= config.max_threads,
            "Thread count should be between min and max");

        // 测试中等数量目录的情况 (3.5 * dirs_per_thread)
        let config = ThreadPoolConfig {
            min_threads: 2,
            max_threads: 8,
            dirs_per_thread: 100,
            auto_adjust: true,
        };
        let pool = AdaptiveThreadPool::new(config.clone());
        pool.update_directory_count(350);
        let threads = pool.adjust_thread_count();
        assert!(threads >= config.min_threads && threads <= config.max_threads,
            "Thread count should be between min and max");

        // 测试大量目录的情况 (10 * dirs_per_thread)
        let config = ThreadPoolConfig {
            min_threads: 2,
            max_threads: 8,
            dirs_per_thread: 100,
            auto_adjust: true,
        };
        let pool = AdaptiveThreadPool::new(config.clone());
        pool.update_directory_count(1000);
        let threads = pool.adjust_thread_count();
        assert!(threads >= config.min_threads && threads <= config.max_threads,
            "Thread count should be between min and max");
    }
    
    #[test]
    fn test_adaptive_thread_pool_no_auto_adjust() {
        let config = ThreadPoolConfig {
            min_threads: 3,
            max_threads: 8,
            dirs_per_thread: 100,
            auto_adjust: false,
        };
        
        let pool = AdaptiveThreadPool::new(config.clone());
        
        // 无论目录数量如何，都应该使用初始线程数
        pool.update_directory_count(1000);
        assert_eq!(pool.adjust_thread_count(), 3);
    }
}