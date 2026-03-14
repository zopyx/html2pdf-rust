#!/usr/bin/env bash
# HTML2PDF Installer Script
# Supports Linux and macOS

set -euo pipefail

# Configuration
REPO="yourusername/html2pdf-rs"
BINARY_NAME="html2pdf"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
VERSION="${VERSION:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    if [[ "${DEBUG:-}" == "true" ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

# Detect architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        armv7l)
            echo "armv7"
            ;;
        i386|i686)
            echo "i686"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Detect OS
detect_os() {
    local os
    os=$(uname -s)
    case "$os" in
        Linux)
            echo "linux"
            ;;
        Darwin)
            echo "macos"
            ;;
        *)
            log_error "Unsupported operating system: $os"
            exit 1
            ;;
    esac
}

# Get latest release version from GitHub
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local version
    
    if command -v curl &> /dev/null; then
        version=$(curl -s "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget &> /dev/null; then
        version=$(wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
    
    echo "$version"
}

# Construct download URL
get_download_url() {
    local version="$1"
    local os="$2"
    local arch="$3"
    local musl=""
    
    # Detect musl libc
    if [[ "$os" == "linux" ]] && ldd --version 2>&1 | grep -qi musl; then
        musl="-musl"
    fi
    
    local target="${os}-${arch}${musl}"
    local package_name="${BINARY_NAME}-${version}-${target}.tar.gz"
    
    echo "https://github.com/${REPO}/releases/download/${version}/${package_name}"
}

# Download file
download() {
    local url="$1"
    local output="$2"
    
    log_info "Downloading from: $url"
    
    if command -v curl &> /dev/null; then
        curl -fsSL --progress-bar "$url" -o "$output"
    elif command -v wget &> /dev/null; then
        wget -q --show-progress "$url" -O "$output"
    else
        log_error "Neither curl nor wget found"
        exit 1
    fi
}

# Verify checksum
verify_checksum() {
    local file="$1"
    local checksum_url="${file}.sha256"
    local checksum_file="${file}.sha256"
    
    if [[ ! -f "$checksum_file" ]]; then
        log_warn "Checksum file not found, skipping verification"
        return 0
    fi
    
    log_info "Verifying checksum..."
    
    if command -v sha256sum &> /dev/null; then
        sha256sum -c "$checksum_file" --quiet
    elif command -v shasum &> /dev/null; then
        shasum -a 256 -c "$checksum_file" --quiet
    else
        log_warn "No checksum tool found, skipping verification"
        return 0
    fi
}

# Install binary
install_binary() {
    local temp_dir="$1"
    local extract_dir="${temp_dir}/extract"
    local archive="${temp_dir}/archive.tar.gz"
    
    log_info "Extracting archive..."
    mkdir -p "$extract_dir"
    tar -xzf "$archive" -C "$extract_dir" --strip-components=1
    
    local binary_path="${extract_dir}/${BINARY_NAME}"
    
    if [[ ! -f "$binary_path" ]]; then
        log_error "Binary not found in archive"
        exit 1
    fi
    
    # Make binary executable
    chmod +x "$binary_path"
    
    # Check if we need sudo
    local use_sudo=""
    if [[ ! -w "$INSTALL_DIR" ]]; then
        use_sudo="sudo"
        log_warn "Installation directory is not writable, using sudo"
    fi
    
    log_info "Installing binary to ${INSTALL_DIR}/${BINARY_NAME}..."
    $use_sudo mkdir -p "$INSTALL_DIR"
    $use_sudo cp "$binary_path" "${INSTALL_DIR}/${BINARY_NAME}"
    
    log_info "Installation complete!"
}

# Post-installation checks
post_install() {
    local installed_path="${INSTALL_DIR}/${BINARY_NAME}"
    
    if [[ ! -f "$installed_path" ]]; then
        log_error "Installation verification failed"
        exit 1
    fi
    
    log_info "Installed version:"
    "$installed_path" --version
    
    # Check if in PATH
    if ! command -v "$BINARY_NAME" &> /dev/null; then
        echo ""
        log_warn "${INSTALL_DIR} is not in your PATH"
        echo "Add the following to your shell configuration:"
        echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

# Cleanup function
cleanup() {
    local temp_dir="$1"
    if [[ -d "$temp_dir" ]]; then
        rm -rf "$temp_dir"
    fi
}

# Print help
print_help() {
    cat <<EOF
HTML2PDF Installer

Usage: install.sh [OPTIONS]

Options:
    -v, --version VERSION   Install specific version (default: latest)
    -d, --dir DIR           Installation directory (default: /usr/local/bin)
    -r, --repo REPO         GitHub repository (default: yourusername/html2pdf-rs)
    --no-checksum           Skip checksum verification
    --debug                 Enable debug output
    -h, --help              Show this help message

Environment Variables:
    INSTALL_DIR             Installation directory
    VERSION                 Version to install
    GITHUB_TOKEN            GitHub token for API rate limits (optional)

Examples:
    ./install.sh                           # Install latest version
    ./install.sh -v v0.1.0                 # Install specific version
    ./install.sh -d ~/.local/bin           # Install to custom directory
    VERSION=v0.1.0 ./install.sh            # Using environment variable

EOF
}

# Main function
main() {
    # Parse arguments
    local skip_checksum=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                VERSION="$2"
                shift 2
                ;;
            -d|--dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            -r|--repo)
                REPO="$2"
                shift 2
                ;;
            --no-checksum)
                skip_checksum=true
                shift
                ;;
            --debug)
                DEBUG=true
                shift
                ;;
            -h|--help)
                print_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_help
                exit 1
                ;;
        esac
    done
    
    echo "========================================"
    echo "  HTML2PDF Installer"
    echo "========================================"
    echo ""
    
    # Detect system
    local os arch
    os=$(detect_os)
    arch=$(detect_arch)
    
    log_info "Detected OS: $os"
    log_info "Detected Architecture: $arch"
    
    # Get version
    if [[ "$VERSION" == "latest" ]]; then
        log_info "Fetching latest version..."
        VERSION=$(get_latest_version)
        if [[ -z "$VERSION" ]]; then
            log_error "Failed to get latest version"
            exit 1
        fi
    fi
    
    log_info "Version: $VERSION"
    
    # Get download URL
    local download_url
    download_url=$(get_download_url "$VERSION" "$os" "$arch")
    log_debug "Download URL: $download_url"
    
    # Create temp directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "cleanup $temp_dir" EXIT
    
    # Download archive
    local archive_path="${temp_dir}/archive.tar.gz"
    log_info "Downloading HTML2PDF..."
    if ! download "$download_url" "$archive_path"; then
        log_error "Download failed"
        log_error "URL: $download_url"
        log_error "This might be because:"
        log_error "  - The version doesn't exist"
        log_error "  - Your platform is not supported"
        log_error "  - Network issues"
        exit 1
    fi
    
    # Download checksum
    if [[ "$skip_checksum" == false ]]; then
        local checksum_url="${download_url}.sha256"
        local checksum_path="${archive_path}.sha256"
        if download "$checksum_url" "$checksum_path" 2>/dev/null; then
            verify_checksum "$archive_path"
        else
            log_warn "Could not download checksum file"
        fi
    fi
    
    # Install
    install_binary "$temp_dir"
    
    # Post-install
    post_install
    
    echo ""
    echo "========================================"
    log_info "Installation successful!"
    echo "========================================"
    echo ""
    echo "Run '${BINARY_NAME} --help' to get started"
}

main "$@"
