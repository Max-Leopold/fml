#!/bin/sh
# Install script for fml (Factorio Mod Loader)
# Usage: curl -fsSL https://raw.githubusercontent.com/Max-Leopold/fml/main/install.sh | sh
#
# Options (via environment variables):
#   FML_VERSION   - Install a specific version (e.g. "v0.0.1"). Default: latest.
#   INSTALL_DIR   - Directory to install to. Default: /usr/local/bin or ~/.local/bin.

set -eu

REPO="Max-Leopold/fml"
BINARY_NAME="fml"
CURL_OPTS="--proto =https --tlsv1.2"

# --- Helpers ---

info() {
    printf '\033[1;34m%s\033[0m\n' "$*"
}

error() {
    printf '\033[1;31merror: %s\033[0m\n' "$*" >&2
    exit 1
}

need() {
    if ! command -v "$1" > /dev/null 2>&1; then
        error "required command '$1' not found"
    fi
}

# --- Detect platform ---

detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Linux)  OS="linux" ;;
        Darwin) OS="darwin" ;;
        *)      error "unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH="x86_64" ;;
        arm64|aarch64)  ARCH="aarch64" ;;
        *)              error "unsupported architecture: $ARCH" ;;
    esac

    case "${OS}-${ARCH}" in
        linux-x86_64)
            TARGET="x86_64-unknown-linux-musl"
            ARCHIVE_EXT="tar.gz"
            ;;
        darwin-x86_64)
            TARGET="x86_64-apple-darwin"
            ARCHIVE_EXT="tar.gz"
            ;;
        darwin-aarch64)
            TARGET="aarch64-apple-darwin"
            ARCHIVE_EXT="tar.gz"
            ;;
        *)
            error "no pre-built binary for ${OS}/${ARCH}"
            ;;
    esac
}

# --- Resolve version ---

get_version() {
    if [ -n "${FML_VERSION:-}" ]; then
        VERSION="$FML_VERSION"
    else
        info "Fetching latest release version..."
        # Follow the redirect from /releases/latest to extract the tag
        REDIRECT_URL=$(curl $CURL_OPTS -fsSI "https://github.com/${REPO}/releases/latest" \
            | grep -i '^location:' \
            | tr -d '\r\n')

        case "$REDIRECT_URL" in
            */tag/*)
                VERSION=$(echo "$REDIRECT_URL" | sed 's|.*/tag/||')
                ;;
            *)
                error "no releases found at https://github.com/${REPO}/releases"
                ;;
        esac

        if [ -z "$VERSION" ]; then
            error "could not determine latest version"
        fi
    fi

    # Validate version format (must start with 'v' followed by a digit)
    case "$VERSION" in
        v[0-9]*)  ;; # valid
        *)        error "invalid version format: '${VERSION}' (expected e.g. 'v0.1.0')" ;;
    esac

    info "Installing fml ${VERSION}"
}

# --- Determine install directory ---

get_install_dir() {
    if [ -n "${INSTALL_DIR:-}" ]; then
        INSTALL_TO="$INSTALL_DIR"
    else
        # Always prefer /usr/local/bin — we'll use sudo later if needed
        INSTALL_TO="/usr/local/bin"
    fi

    # Create dir if it doesn't exist (may need sudo)
    if ! [ -d "$INSTALL_TO" ]; then
        if [ "$(id -u)" -ne 0 ] && ! mkdir -p "$INSTALL_TO" 2>/dev/null; then
            # Can't create preferred dir without sudo, fall back
            if [ -z "${HOME:-}" ]; then
                error "cannot fall back to ~/.local/bin: HOME is not set"
            fi
            INSTALL_TO="${HOME}/.local/bin"
            mkdir -p "$INSTALL_TO"
        fi
    fi
}

# --- Download and install ---

download_and_install() {
    work_dir=$(mktemp -d)
    trap 'rm -rf "$work_dir"' EXIT INT TERM

    ARCHIVE_NAME="${BINARY_NAME}-${TARGET}.${ARCHIVE_EXT}"
    ARCHIVE_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"

    info "Downloading ${ARCHIVE_NAME}..."
    curl $CURL_OPTS -fSL --progress-bar -o "${work_dir}/${ARCHIVE_NAME}" "$ARCHIVE_URL" \
        || error "download failed — does release ${VERSION} have a ${TARGET} binary?"

    # Extract
    info "Extracting..."
    tar xzf "${work_dir}/${ARCHIVE_NAME}" -C "$work_dir"

    # Find the binary
    BINARY_PATH=""
    for name in "$BINARY_NAME" "factorio-mod-loader"; do
        if [ -f "${work_dir}/${name}" ]; then
            BINARY_PATH="${work_dir}/${name}"
            break
        fi
    done

    if [ -z "$BINARY_PATH" ]; then
        error "binary not found in archive (expected '${BINARY_NAME}' or 'factorio-mod-loader')"
    fi

    # Install
    chmod +x "$BINARY_PATH"
    if [ "$(id -u)" -ne 0 ] && ! [ -w "$INSTALL_TO" ]; then
        if [ ! -t 0 ]; then
            error "need sudo to install to ${INSTALL_TO}, but running non-interactively. Either:\n  - Re-run with: curl -fsSL ... | sudo sh\n  - Or set INSTALL_DIR: INSTALL_DIR=~/.local/bin curl -fsSL ... | sh"
        fi
        info "Installing to ${INSTALL_TO} (requires sudo)..."
        sudo install -m 755 "$BINARY_PATH" "${INSTALL_TO}/${BINARY_NAME}"
    else
        install -m 755 "$BINARY_PATH" "${INSTALL_TO}/${BINARY_NAME}"
    fi
}

# --- Post-install check ---

post_install() {
    info "Installed fml to ${INSTALL_TO}/${BINARY_NAME}"

    # Check if install dir is in PATH
    case ":${PATH}:" in
        *":${INSTALL_TO}:"*)
            info "Done! Run 'fml --help' to get started."
            ;;
        *)
            printf '\n'
            info "WARNING: ${INSTALL_TO} is not in your PATH."
            info "Add it by running:"
            printf '\n'
            printf '  export PATH="%s:$PATH"\n' "$INSTALL_TO"
            printf '\n'
            info "Add the line above to your ~/.bashrc or ~/.profile to make it permanent."
            ;;
    esac
}

# --- Main ---

main() {
    need curl
    need tar

    detect_platform
    get_version
    get_install_dir
    download_and_install
    post_install
}

main
