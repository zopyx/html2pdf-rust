#!/usr/bin/env bash
# Package script for html2pdf-rs
# Creates release packages for different platforms

set -euo pipefail

# Configuration
PACKAGE_NAME="html2pdf"
VERSION="${VERSION:-$(cargo metadata --no-deps --format-version 1 | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)}"
TARGET="${TARGET:-$(rustc -vV | sed -n 's|host: ||p')}"
BUILD_DIR="target/package"
DIST_DIR="dist"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Print usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Create release packages for html2pdf

OPTIONS:
    -t, --target TARGET     Target triple (default: host target)
    -v, --version VERSION   Version override
    -o, --output DIR        Output directory (default: dist)
    --tar                   Create tar.gz package
    --zip                   Create zip package
    --deb                   Create Debian package (requires cargo-deb)
    --rpm                   Create RPM package (requires cargo-generate-rpm)
    --all                   Create all package types
    -h, --help              Show this help message

EXAMPLES:
    $0                      # Create default package for current platform
    $0 --tar --zip          # Create both tar.gz and zip packages
    $0 --all                # Create all package types
    $0 -t x86_64-unknown-linux-musl --tar

EOF
}

# Parse arguments
CREATE_TAR=false
CREATE_ZIP=false
CREATE_DEB=false
CREATE_RPM=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target)
            TARGET="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -o|--output)
            DIST_DIR="$2"
            shift 2
            ;;
        --tar)
            CREATE_TAR=true
            shift
            ;;
        --zip)
            CREATE_ZIP=true
            shift
            ;;
        --deb)
            CREATE_DEB=true
            shift
            ;;
        --rpm)
            CREATE_RPM=true
            shift
            ;;
        --all)
            CREATE_TAR=true
            CREATE_ZIP=true
            CREATE_DEB=true
            CREATE_RPM=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Default to tar if no package type specified
if [[ "$CREATE_TAR" == false && "$CREATE_ZIP" == false && "$CREATE_DEB" == false && "$CREATE_RPM" == false ]]; then
    CREATE_TAR=true
fi

# Normalize target name for package naming
normalize_target_name() {
    local target="$1"
    case "$target" in
        x86_64-unknown-linux-gnu)    echo "linux-x86_64" ;;
        x86_64-unknown-linux-musl)   echo "linux-x86_64-musl" ;;
        aarch64-unknown-linux-gnu)   echo "linux-aarch64" ;;
        aarch64-unknown-linux-musl)  echo "linux-aarch64-musl" ;;
        x86_64-apple-darwin)         echo "macos-x86_64" ;;
        aarch64-apple-darwin)        echo "macos-aarch64" ;;
        x86_64-pc-windows-msvc)      echo "windows-x86_64" ;;
        x86_64-pc-windows-gnu)       echo "windows-x86_64-gnu" ;;
        i686-pc-windows-msvc)        echo "windows-i686" ;;
        *)                           echo "$target" ;;
    esac
}

PACKAGE_TARGET=$(normalize_target_name "$TARGET")
PACKAGE_BASENAME="${PACKAGE_NAME}-v${VERSION}-${PACKAGE_TARGET}"

cleanup() {
    if [[ -d "$BUILD_DIR" ]]; then
        log_info "Cleaning up build directory..."
        rm -rf "$BUILD_DIR"
    fi
}

trap cleanup EXIT

# Build the release binary
build_release() {
    log_info "Building release binary for target: $TARGET"
    
    if command -v cross &> /dev/null; then
        log_info "Using cross for cross-compilation"
        cross build --release --target "$TARGET"
    else
        if [[ "$TARGET" != "$(rustc -vV | sed -n 's|host: ||p')" ]]; then
            log_warn "Cross-compilation requested but 'cross' not installed"
            log_warn "Installing cross..."
            cargo install cross --git https://github.com/cross-rs/cross
            cross build --release --target "$TARGET"
        else
            cargo build --release --target "$TARGET"
        fi
    fi
}

# Create staging directory
prepare_package() {
    log_info "Preparing package contents..."
    
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR/$PACKAGE_BASENAME"
    
    # Copy binary
    local binary_name="html2pdf"
    if [[ "$TARGET" == *"windows"* ]]; then
        binary_name="html2pdf.exe"
    fi
    
    cp "target/$TARGET/release/$binary_name" "$BUILD_DIR/$PACKAGE_BASENAME/"
    
    # Copy documentation
    cp README.md "$BUILD_DIR/$PACKAGE_BASENAME/" 2>/dev/null || true
    cp LICENSE* "$BUILD_DIR/$PACKAGE_BASENAME/" 2>/dev/null || true
    cp CHANGELOG.md "$BUILD_DIR/$PACKAGE_BASENAME/" 2>/dev/null || true
    
    # Copy installation scripts
    if [[ "$TARGET" == *"linux"* || "$TARGET" == *"darwin"* ]]; then
        mkdir -p "$BUILD_DIR/$PACKAGE_BASENAME/scripts"
        cp scripts/install.sh "$BUILD_DIR/$PACKAGE_BASENAME/scripts/" 2>/dev/null || true
    fi
    
    if [[ "$TARGET" == *"windows"* ]]; then
        mkdir -p "$BUILD_DIR/$PACKAGE_BASENAME/scripts"
        cp scripts/install.ps1 "$BUILD_DIR/$PACKAGE_BASENAME/scripts/" 2>/dev/null || true
    fi
}

# Create tar.gz package
create_tar_package() {
    log_info "Creating tar.gz package..."
    
    mkdir -p "$DIST_DIR"
    
    cd "$BUILD_DIR"
    tar czf "../$DIST_DIR/${PACKAGE_BASENAME}.tar.gz" "$PACKAGE_BASENAME"
    cd - > /dev/null
    
    # Create checksum
    cd "$DIST_DIR"
    sha256sum "${PACKAGE_BASENAME}.tar.gz" > "${PACKAGE_BASENAME}.tar.gz.sha256"
    cd - > /dev/null
    
    log_info "Created: $DIST_DIR/${PACKAGE_BASENAME}.tar.gz"
}

# Create zip package
create_zip_package() {
    log_info "Creating zip package..."
    
    mkdir -p "$DIST_DIR"
    
    cd "$BUILD_DIR"
    if command -v zip &> /dev/null; then
        zip -r "../$DIST_DIR/${PACKAGE_BASENAME}.zip" "$PACKAGE_BASENAME"
    else
        log_warn "zip command not found, trying alternative method..."
        python3 -c "import shutil; shutil.make_archive('../$DIST_DIR/${PACKAGE_BASENAME}', 'zip', '.', '$PACKAGE_BASENAME')"
    fi
    cd - > /dev/null
    
    # Create checksum
    cd "$DIST_DIR"
    sha256sum "${PACKAGE_BASENAME}.zip" > "${PACKAGE_BASENAME}.zip.sha256"
    cd - > /dev/null
    
    log_info "Created: $DIST_DIR/${PACKAGE_BASENAME}.zip"
}

# Create Debian package
create_deb_package() {
    log_info "Creating Debian package..."
    
    if ! command -v cargo-deb &> /dev/null; then
        log_warn "cargo-deb not installed, skipping Debian package"
        log_info "Install with: cargo install cargo-deb"
        return
    fi
    
    mkdir -p "$DIST_DIR"
    
    if [[ -n "${TARGET:-}" ]]; then
        cargo deb --target "$TARGET" --output "$DIST_DIR/${PACKAGE_BASENAME}.deb"
    else
        cargo deb --output "$DIST_DIR/${PACKAGE_BASENAME}.deb"
    fi
    
    # Create checksum
    cd "$DIST_DIR"
    sha256sum "${PACKAGE_BASENAME}.deb" > "${PACKAGE_BASENAME}.deb.sha256"
    cd - > /dev/null
    
    log_info "Created: $DIST_DIR/${PACKAGE_BASENAME}.deb"
}

# Create RPM package
create_rpm_package() {
    log_info "Creating RPM package..."
    
    if ! command -v cargo-generate-rpm &> /dev/null; then
        log_warn "cargo-generate-rpm not installed, skipping RPM package"
        log_info "Install with: cargo install cargo-generate-rpm"
        return
    fi
    
    mkdir -p "$DIST_DIR"
    
    if [[ -n "${TARGET:-}" ]]; then
        cargo generate-rpm --target "$TARGET" --output "$DIST_DIR/${PACKAGE_BASENAME}.rpm"
    else
        cargo generate-rpm --output "$DIST_DIR/${PACKAGE_BASENAME}.rpm"
    fi
    
    # Create checksum
    cd "$DIST_DIR"
    sha256sum "${PACKAGE_BASENAME}.rpm" > "${PACKAGE_BASENAME}.rpm.sha256"
    cd - > /dev/null
    
    log_info "Created: $DIST_DIR/${PACKAGE_BASENAME}.rpm"
}

# Main execution
main() {
    log_info "Packaging $PACKAGE_NAME v$VERSION for $TARGET"
    
    # Build release
    build_release
    
    # Prepare package contents
    prepare_package
    
    # Create requested packages
    if [[ "$CREATE_TAR" == true ]]; then
        create_tar_package
    fi
    
    if [[ "$CREATE_ZIP" == true ]]; then
        create_zip_package
    fi
    
    if [[ "$CREATE_DEB" == true ]]; then
        create_deb_package
    fi
    
    if [[ "$CREATE_RPM" == true ]]; then
        create_rpm_package
    fi
    
    log_info "Packaging complete!"
    log_info "Output directory: $DIST_DIR"
    
    # List created packages
    if [[ -d "$DIST_DIR" ]]; then
        echo ""
        ls -lh "$DIST_DIR"/*.tar.gz "$DIST_DIR"/*.zip "$DIST_DIR"/*.deb "$DIST_DIR"/*.rpm 2>/dev/null || true
    fi
}

main "$@"
