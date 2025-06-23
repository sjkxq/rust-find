# Rust Find

一个高性能的文件搜索工具，使用 Rust 实现。它提供了类似于 Linux `find` 命令的功能，但支持并行搜索和更多现代特性。

## 特性

- 🚀 **高性能并行搜索**：利用多线程加速大型目录的搜索
- 🎯 **灵活的过滤系统**：支持多种文件过滤条件
  - 文件名匹配（支持通配符）
  - 大小写敏感/不敏感搜索
  - 文件类型过滤
- 🔄 **可配置的搜索选项**
  - 最大搜索深度限制
  - 符号链接跟随选项
  - 路径格式选项（绝对/相对路径）
- 🛡️ **健壮的错误处理**：详细的错误报告和日志记录
- 🎨 **用户友好的界面**：直观的命令行接口

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/rust-find.git
cd rust-find

# 构建项目
cargo build --release

# 运行测试
cargo test
```

### 使用 Cargo 安装

```bash
cargo install rust-find
```

## 使用示例

### 基本用法

```bash
# 在当前目录中查找所有文件
rust-find .

# 在指定目录中查找文件
rust-find /path/to/search
```

### 使用过滤器

```bash
# 查找所有 .rs 文件
rust-find . --name "*.rs"

# 不区分大小写查找
rust-find . --name "*.RS" --ignore-case

# 限制搜索深度
rust-find . --maxdepth 2
```

### 高级选项

```bash
# 使用并行搜索（默认启用）
rust-find . --parallel

# 跟随符号链接
rust-find . --follow

# 显示绝对路径
rust-find . --absolute-path

# 显示相对路径
rust-find . --relative-path
```

## 命令行选项

```
USAGE:
    rust-find [OPTIONS] <PATH>...

OPTIONS:
    -n, --name <PATTERN>      按文件名匹配（支持通配符）
    -i, --ignore-case         不区分大小写匹配
    -d, --maxdepth <DEPTH>    最大搜索深度
    -f, --follow             跟随符号链接
    -a, --absolute-path      显示绝对路径
    -r, --relative-path      显示相对路径
    -p, --parallel           启用并行搜索（默认：true）
    -h, --help              显示帮助信息
    -V, --version           显示版本信息
```

## 项目结构

```
rust-find/
├── src/
│   ├── main.rs           # 程序入口点
│   ├── lib.rs            # 库入口点
│   ├── cli.rs            # 命令行接口
│   ├── errors.rs         # 错误处理
│   └── finder/           # 核心查找功能
│       ├── mod.rs        # 模块定义
│       ├── filter.rs     # 文件过滤器
│       ├── options.rs    # 查找选项
│       └── walker.rs     # 文件系统遍历
```

## 开发指南

### 构建要求

- Rust 1.56.0 或更高版本
- Cargo（Rust 包管理器）

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行基准测试
cargo bench
```

### 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request