#!/bin/sh
# GhostFS Installer for macOS / Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/ShejanMahamud/ghostfs/main/install.sh | sh

set -e

REPO="ShejanMahamud/ghostfs"
BINARY_NAME="ghost"
INSTALL_DIR="$HOME/.ghostfs/bin"

echo ""
echo "  👻 GhostFS Installer"
echo "  ====================="
echo ""

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  TARGET_OS="unknown-linux-gnu" ;;
    Darwin) TARGET_OS="apple-darwin" ;;
    *)      echo "  Error: Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64)  TARGET_ARCH="x86_64" ;;
    aarch64) TARGET_ARCH="aarch64" ;;
    arm64)   TARGET_ARCH="aarch64" ;;
    *)       echo "  Error: Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"
ASSET_NAME="ghost-${TARGET}.tar.gz"

# Fetch latest release
echo "  Fetching latest release..."
RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"

if command -v curl > /dev/null 2>&1; then
    RELEASE_JSON=$(curl -fsSL "$RELEASE_URL")
elif command -v wget > /dev/null 2>&1; then
    RELEASE_JSON=$(wget -qO- "$RELEASE_URL")
else
    echo "  Error: curl or wget required"
    exit 1
fi

VERSION=$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
DOWNLOAD_URL=$(echo "$RELEASE_JSON" | grep "browser_download_url.*${ASSET_NAME}" | head -1 | sed 's/.*"\(https[^"]*\)".*/\1/')

if [ -z "$DOWNLOAD_URL" ]; then
    echo "  Error: No binary found for $TARGET"
    exit 1
fi

echo "  Version:  $VERSION"
echo "  Platform: $TARGET"
echo ""

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download and extract
TEMP_FILE=$(mktemp)
echo "  Downloading ${ASSET_NAME}..."

if command -v curl > /dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE"
else
    wget -q "$DOWNLOAD_URL" -O "$TEMP_FILE"
fi

echo "  Extracting..."
tar -xzf "$TEMP_FILE" -C "$INSTALL_DIR"
rm -f "$TEMP_FILE"

# Make executable
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Verify
if [ ! -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    echo "  Error: Binary not found after extraction"
    exit 1
fi

# Add to PATH
SHELL_NAME=$(basename "$SHELL")
PROFILE=""

case "$SHELL_NAME" in
    zsh)  PROFILE="$HOME/.zshrc" ;;
    bash) PROFILE="$HOME/.bashrc" ;;
    fish) PROFILE="$HOME/.config/fish/config.fish" ;;
    *)    PROFILE="$HOME/.profile" ;;
esac

if [ -n "$PROFILE" ] && [ -f "$PROFILE" ]; then
    if ! grep -q "\.ghostfs/bin" "$PROFILE" 2>/dev/null; then
        echo "" >> "$PROFILE"
        echo "# GhostFS" >> "$PROFILE"
        echo 'export PATH="$HOME/.ghostfs/bin:$PATH"' >> "$PROFILE"
        echo "  Added ~/.ghostfs/bin to PATH in $PROFILE"
    fi
fi

export PATH="$INSTALL_DIR:$PATH"

echo ""
echo "  ✅ GhostFS $VERSION installed successfully!"
echo ""
echo "  Location: $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "  Get started:"
echo "    ghost init"
echo "    ghost add react"
echo "    ghost install"
echo ""
echo "  Run 'ghost --help' for all commands."
echo ""

if [ "$SHELL_NAME" != "fish" ]; then
    echo "  ⚠  Restart your terminal or run:"
    echo "     export PATH=\"\$HOME/.ghostfs/bin:\$PATH\""
    echo ""
fi
