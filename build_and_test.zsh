#!/usr/bin/env zsh
# 快速构建和测试 rust-find 项目的脚本
# 作者: Craft AI Assistant

# 设置严格模式
set -e  # 遇到错误立即退出
set -u  # 使用未定义的变量时报错

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # 无颜色

# 默认参数
BUILD_TYPE="debug"
RUN_TESTS=true
TEST_FILTER=""
VERBOSE=false
CLEAN=false

# 显示帮助信息
function show_help() {
    echo "${BOLD}用法:${NC} $0 [选项]"
    echo
    echo "快速构建 rust-find 项目并运行测试的脚本"
    echo
    echo "${BOLD}选项:${NC}"
    echo "  -h, --help        显示此帮助信息并退出"
    echo "  -r, --release     使用发布模式构建"
    echo "  -b, --build-only  仅构建项目，不运行测试"
    echo "  -t, --test-only   仅运行测试，不构建项目"
    echo "  -f, --filter      指定测试过滤器 (例如: -f 'name_filter')"
    echo "  -v, --verbose     显示详细输出"
    echo "  -c, --clean       在构建前清理项目"
    echo
    echo "${BOLD}示例:${NC}"
    echo "  $0                       # 构建项目并运行所有测试"
    echo "  $0 -r                    # 使用发布模式构建并测试"
    echo "  $0 -b                    # 仅构建项目"
    echo "  $0 -t -f 'name_filter'   # 仅运行名称包含 'name_filter' 的测试"
    echo "  $0 -c -v                 # 清理项目，构建并测试，显示详细输出"
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -r|--release)
            BUILD_TYPE="release"
            shift
            ;;
        -b|--build-only)
            RUN_TESTS=false
            shift
            ;;
        -t|--test-only)
            BUILD_TYPE=""
            shift
            ;;
        -f|--filter)
            if [[ $# -lt 2 ]]; then
                echo "${RED}错误: --filter 选项需要一个参数${NC}" >&2
                exit 1
            fi
            TEST_FILTER=$2
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        *)
            echo "${RED}错误: 未知选项 $1${NC}" >&2
            show_help
            exit 1
            ;;
    esac
done

# 显示标题
function print_header() {
    echo "${BLUE}${BOLD}$1${NC}"
    echo "${BLUE}${BOLD}$(printf '=%.0s' {1..50})${NC}"
}

# 显示成功消息
function print_success() {
    echo "${GREEN}${BOLD}✓ $1${NC}"
}

# 显示错误消息
function print_error() {
    echo "${RED}${BOLD}✗ $1${NC}" >&2
}

# 显示警告消息
function print_warning() {
    echo "${YELLOW}${BOLD}! $1${NC}" >&2
}

# 显示信息消息
function print_info() {
    echo "${BOLD}$1${NC}"
}

# 检查 Cargo.toml 是否存在
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Cargo.toml 文件未找到。请确保你在项目根目录中运行此脚本。"
    exit 1
fi

# 清理项目
if [[ "$CLEAN" = true ]]; then
    print_header "清理项目"
    cargo clean
    if [[ $? -eq 0 ]]; then
        print_success "项目清理完成"
    else
        print_error "项目清理失败"
        exit 1
    fi
fi

# 构建项目
if [[ -n "$BUILD_TYPE" ]]; then
    if [[ "$BUILD_TYPE" = "release" ]]; then
        print_header "构建项目 (发布模式)"
        BUILD_CMD="cargo build --release"
    else
        print_header "构建项目 (调试模式)"
        BUILD_CMD="cargo build"
    fi
    
    if [[ "$VERBOSE" = true ]]; then
        BUILD_CMD="$BUILD_CMD --verbose"
    fi
    
    eval $BUILD_CMD
    
    if [[ $? -eq 0 ]]; then
        print_success "构建成功"
    else
        print_error "构建失败"
        exit 1
    fi
fi

# 运行测试
if [[ "$RUN_TESTS" = true ]]; then
    print_header "运行测试"
    
    TEST_CMD="cargo test"
    
    if [[ -n "$TEST_FILTER" ]]; then
        print_info "使用过滤器: $TEST_FILTER"
        TEST_CMD="$TEST_CMD $TEST_FILTER"
    fi
    
    if [[ "$BUILD_TYPE" = "release" ]]; then
        TEST_CMD="$TEST_CMD --release"
    fi
    
    if [[ "$VERBOSE" = true ]]; then
        TEST_CMD="$TEST_CMD --verbose"
    fi
    
    eval $TEST_CMD
    
    if [[ $? -eq 0 ]]; then
        print_success "所有测试通过"
    else
        print_error "测试失败"
        exit 1
    fi
fi

# 显示完成消息
if [[ -n "$BUILD_TYPE" && "$RUN_TESTS" = true ]]; then
    print_header "构建和测试完成"
elif [[ -n "$BUILD_TYPE" ]]; then
    print_header "构建完成"
else
    print_header "测试完成"
fi

# 如果是发布模式，显示二进制文件位置
if [[ "$BUILD_TYPE" = "release" ]]; then
    BINARY_PATH="./target/release/rust-find"
    if [[ -f "$BINARY_PATH" ]]; then
        print_info "发布二进制文件位置: $BINARY_PATH"
        print_info "运行示例: $BINARY_PATH --name \"*.rs\" ."
    fi
elif [[ -n "$BUILD_TYPE" ]]; then
    BINARY_PATH="./target/debug/rust-find"
    if [[ -f "$BINARY_PATH" ]]; then
        print_info "调试二进制文件位置: $BINARY_PATH"
        print_info "运行示例: $BINARY_PATH --name \"*.rs\" ."
    fi
fi

exit 0