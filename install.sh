#!/bin/bash
set -e

REPO="TrustDec/proxydoc"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  linux) TARGET="${ARCH}-unknown-linux-gnu" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

echo "🩺 Installing proxydoc for $TARGET..."

LATEST_RELEASE=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$LATEST_RELEASE" ]; then
  LATEST_RELEASE="v0.1.0"
fi

URL="https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/proxydoc-${TARGET}.tar.gz"

TMP_DIR=$(mktemp -d)
curl -fsSL "$URL" -o "$TMP_DIR/proxydoc.tar.gz"
tar -xzf "$TMP_DIR/proxydoc.tar.gz" -C "$TMP_DIR"

INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  echo "Installing to $INSTALL_DIR (requires sudo)..."
  sudo mv "$TMP_DIR/proxydoc" "$INSTALL_DIR/"
else
  mv "$TMP_DIR/proxydoc" "$INSTALL_DIR/"
fi

rm -rf "$TMP_DIR"
echo "✅ proxydoc successfully installed to $INSTALL_DIR/proxydoc"
echo "Run 'proxydoc doctor' to get started!"
