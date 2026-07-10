#!/usr/bin/env bash
set -euo pipefail

echo "  📄 PaperCoat Installer"
echo ""

# Quick path: Homebrew on macOS
if [[ "$(uname)" == "Darwin" ]] && command -v brew &>/dev/null; then
    echo "  → Installing via Homebrew..."
    brew install malamutes/tap/papercoat
    exit 0
fi

REPO="malamutes/papercoat"
VERSION="${1:-latest}"
INSTALL_DIR="${PAPERCOAT_INSTALL:-$HOME/.local/bin}"

if [[ "$VERSION" == "latest" ]]; then
    VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    echo "  → Latest version: $VERSION"
fi

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux)  TARGET_OS="unknown-linux-gnu" ;;
    Darwin) TARGET_OS="apple-darwin" ;;
    *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64) TARGET_ARCH="x86_64" ;;
    aarch64|arm64) TARGET_ARCH="aarch64" ;;
    *)            echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"
ARCHIVE="papercoat-${TARGET}.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ARCHIVE"

echo "  → Downloading $URL"

mkdir -p "$INSTALL_DIR"
TMP=$(mktemp -d)
curl -sL "$URL" | tar xz -C "$TMP"
mv "$TMP/papercoat" "$INSTALL_DIR/papercoat"
rm -rf "$TMP"
chmod +x "$INSTALL_DIR/papercoat"

echo "  ✓ Installed papercoat $VERSION to $INSTALL_DIR/papercoat"

if ! echo ":$PATH:" | grep -q ":$INSTALL_DIR:" ; then
    echo "  ⚠  $INSTALL_DIR is not in your PATH"
    echo "     Add to your shell rc: export PATH=\"\$PATH:$INSTALL_DIR\""
fi
