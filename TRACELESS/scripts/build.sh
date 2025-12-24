#!/bin/bash

# Traceless 跨平台构建脚本
# Cross-platform build script for Traceless

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Traceless Build Script${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}[*]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

# 检测操作系统
detect_os() {
    case "$(uname -s)" in
        Darwin*)    OS="macos" ;;
        Linux*)     OS="linux" ;;
        MINGW*|MSYS*|CYGWIN*)    OS="windows" ;;
        *)          OS="unknown" ;;
    esac
    echo $OS
}

# 检测架构
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)    ARCH="x64" ;;
        aarch64|arm64)   ARCH="arm64" ;;
        *)               ARCH="unknown" ;;
    esac
    echo $ARCH
}

# 检查依赖
check_dependencies() {
    print_step "Checking dependencies..."

    # 检查 Node.js
    if ! command -v node &> /dev/null; then
        print_error "Node.js is not installed. Please install Node.js 18+ first."
        exit 1
    fi
    print_success "Node.js $(node --version) found"

    # 检查 npm
    if ! command -v npm &> /dev/null; then
        print_error "npm is not installed."
        exit 1
    fi
    print_success "npm $(npm --version) found"

    # 检查 Rust
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed. Please install Rust first:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    print_success "Rust $(rustc --version | cut -d' ' -f2) found"

    # 检查 Cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed."
        exit 1
    fi
    print_success "Cargo found"
}

# 安装 Linux 依赖
install_linux_deps() {
    print_step "Installing Linux dependencies..."

    if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        sudo apt-get update
        sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            build-essential \
            curl \
            wget \
            file \
            libxdo-dev \
            libssl-dev
        print_success "Debian/Ubuntu dependencies installed"
    elif command -v dnf &> /dev/null; then
        # Fedora
        sudo dnf install -y \
            webkit2gtk4.1-devel \
            libappindicator-gtk3-devel \
            librsvg2-devel \
            openssl-devel \
            patchelf
        print_success "Fedora dependencies installed"
    elif command -v pacman &> /dev/null; then
        # Arch Linux
        sudo pacman -Syu --noconfirm \
            webkit2gtk-4.1 \
            libappindicator-gtk3 \
            librsvg \
            openssl \
            patchelf
        print_success "Arch Linux dependencies installed"
    else
        print_warning "Could not detect package manager. Please install dependencies manually."
    fi
}

# 构建应用
build_app() {
    local os=$(detect_os)
    local arch=$(detect_arch)

    print_step "Building for $os ($arch)..."

    # 安装 npm 依赖
    print_step "Installing npm dependencies..."
    npm ci || npm install

    # 根据平台构建
    case "$os" in
        macos)
            if [ "$arch" = "arm64" ]; then
                print_step "Building for macOS ARM64 (Apple Silicon)..."
                npm run tauri build -- --target aarch64-apple-darwin
            else
                print_step "Building for macOS x64 (Intel)..."
                npm run tauri build -- --target x86_64-apple-darwin
            fi

            # 如果需要同时构建两个架构
            if [ "$1" = "--universal" ]; then
                print_step "Building universal binary..."
                npm run tauri build -- --target aarch64-apple-darwin
                npm run tauri build -- --target x86_64-apple-darwin
            fi
            ;;
        linux)
            print_step "Building for Linux x64..."
            npm run tauri build
            ;;
        windows)
            print_step "Building for Windows x64..."
            npm run tauri build
            ;;
        *)
            print_error "Unsupported operating system: $os"
            exit 1
            ;;
    esac

    print_success "Build completed!"
}

# 显示构建结果
show_results() {
    local os=$(detect_os)

    echo ""
    print_step "Build artifacts:"
    echo ""

    case "$os" in
        macos)
            echo "  DMG installers:"
            find src-tauri/target -name "*.dmg" 2>/dev/null | while read f; do
                echo "    - $f ($(du -h "$f" | cut -f1))"
            done
            echo ""
            echo "  App bundles:"
            find src-tauri/target -name "*.app" -type d 2>/dev/null | while read f; do
                echo "    - $f"
            done
            ;;
        linux)
            echo "  DEB packages:"
            find src-tauri/target -name "*.deb" 2>/dev/null | while read f; do
                echo "    - $f ($(du -h "$f" | cut -f1))"
            done
            echo ""
            echo "  AppImage:"
            find src-tauri/target -name "*.AppImage" 2>/dev/null | while read f; do
                echo "    - $f ($(du -h "$f" | cut -f1))"
            done
            echo ""
            echo "  RPM packages:"
            find src-tauri/target -name "*.rpm" 2>/dev/null | while read f; do
                echo "    - $f ($(du -h "$f" | cut -f1))"
            done
            ;;
        windows)
            echo "  MSI installers:"
            find src-tauri/target -name "*.msi" 2>/dev/null | while read f; do
                echo "    - $f"
            done
            echo ""
            echo "  NSIS installers:"
            find src-tauri/target -name "*-setup.exe" 2>/dev/null | while read f; do
                echo "    - $f"
            done
            ;;
    esac

    echo ""
}

# 主函数
main() {
    print_header

    local os=$(detect_os)
    local arch=$(detect_arch)

    echo "Detected: $os ($arch)"
    echo ""

    check_dependencies

    # Linux 需要额外依赖
    if [ "$os" = "linux" ]; then
        install_linux_deps
    fi

    build_app "$@"
    show_results

    echo ""
    print_success "All done! Check the artifacts above."
}

# 运行
main "$@"
