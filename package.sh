#!/usr/bin/env bash
set -euo pipefail

# 配置
PROJECT_NAME="rust-db-compare"  # 替换为你的项目名称
OUTPUT_DIR="./target/dist"

# rustup target list 获取支持的目标平台
TARGETS=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
    "x86_64-pc-windows-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
)

# 检查是否安装了必要的工具
check_dependencies() {
    echo "检查依赖..."

    # 检查Rust是否安装
    if ! command -v cargo &> /dev/null; then
        echo "未找到cargo，正在安装Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi

    # 创建输出目录
    mkdir -p "$OUTPUT_DIR"
}

# 安装目标平台
install_targets() {
    echo "安装目标平台..."
    for target in "${TARGETS[@]}"; do
        rustup target add "$target" || true
    done
}

# 构建函数
build_target() {
    local target=$1
    echo "开始构建 $target..."

    # 使用cargo进行交叉编译
    rustup target add "$target" || true
    cargo build --release --target "$target" --verbose

    # 确定输出文件名
    local output_name
    local extension=""

    if [[ $target == *-windows-* ]]; then
        extension=".exe"
    fi

    output_name="${PROJECT_NAME}-${target}${extension}"

    # 复制二进制文件到输出目录
    cp "target/$target/release/${PROJECT_NAME}${extension}" \
       "$OUTPUT_DIR/$output_name"

    echo "构建完成: $OUTPUT_DIR/$output_name"
}

# 主函数
main() {
    echo "===== 开始跨平台构建 $PROJECT_NAME ====="

    check_dependencies
    install_targets

    # 为每个目标平台构建
    for target in "${TARGETS[@]}"; do
        build_target "$target"
    done

    echo "===== 所有构建完成 ====="
    echo "输出文件位于: $OUTPUT_DIR"
}

# 运行主函数
main