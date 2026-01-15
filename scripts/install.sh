#!/usr/bin/env sh
set -e

REPO="HomyeeKing/linguist"

# Get version: prefer env var, otherwise fetch from GitHub Releases latest
if [ -n "$LINGUISTO_VERSION" ]; then
  VERSION="$LINGUISTO_VERSION"
else
  echo "Fetching latest version from GitHub releases..."
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | sed -n 's/  \"tag_name\": \"\(.*\)\",/\1/p' | head -n 1)
  if [ -z "$VERSION" ]; then
    echo "Failed to fetch latest version from GitHub, please set LINGUISTO_VERSION manually and retry" 1>&2
    exit 1
  fi
fi

uname_s=$(uname -s)
uname_m=$(uname -m)

case "$uname_s" in
  Linux)  os="unknown-linux-gnu" ;;
  Darwin) os="apple-darwin" ;;
  *)
    echo "Unsupported OS: $uname_s" >&2
    exit 1
    ;;
esac

case "$uname_m" in
  x86_64|amd64) arch="x86_64" ;;
  arm64|aarch64) arch="aarch64" ;;
  *)
    echo "Unsupported architecture: $uname_m" >&2
    exit 1
    ;;
esac

TAR_NAME="linguist-${VERSION}-${arch}-${os}.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
URL="${BASE_URL}/${TAR_NAME}"

INSTALL_DIR="${LINGUISTO_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading $URL ..."
curl -fL "$URL" -o "$TMP_DIR/$TAR_NAME"

cd "$TMP_DIR"
tar -xzf "$TAR_NAME"

if [ ! -f linguist ]; then
  echo "linguist executable not found after extraction" >&2
  exit 1
fi

chmod +x linguist
mv linguist "$INSTALL_DIR/linguist"

echo "linguist installed to $INSTALL_DIR/linguist"

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    echo "You can now run: linguist" ;;
  *)
    echo "Note: $INSTALL_DIR is not in your PATH. Please add it manually:" >&2
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" >&2 ;;
esac
