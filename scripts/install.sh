#!/usr/bin/env sh
set -e

REPO="HomyeeKing/linguist"

# 获取版本：优先使用环境变量，其次自动从 GitHub Releases latest 获取
if [ -n "$LINGUISTO_VERSION" ]; then
  VERSION="$LINGUISTO_VERSION"
else
  echo "Fetching latest version from GitHub releases..."
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | sed -n 's/  \"tag_name\": \"\(.*\)\",/\1/p' | head -n 1)
  if [ -z "$VERSION" ]; then
    echo "无法从 GitHub 获取最新版本号，请设置环境变量 LINGUISTO_VERSION 后重试" 1>&2
    exit 1
  fi
fi

uname_s=$(uname -s)
uname_m=$(uname -m)

case "$uname_s" in
  Linux)  os="unknown-linux-gnu" ;;
  Darwin) os="apple-darwin" ;;
  *)
    echo "不支持的系统: $uname_s" >&2
    exit 1
    ;;
esac

case "$uname_m" in
  x86_64|amd64) arch="x86_64" ;;
  arm64|aarch64) arch="aarch64" ;;
  *)
    echo "不支持的架构: $uname_m" >&2
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
  echo "解压后未找到 linguist 可执行文件" >&2
  exit 1
fi

chmod +x linguist
mv linguist "$INSTALL_DIR/linguist"

echo "linguist 已安装到 $INSTALL_DIR/linguist"

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    echo "你可以直接运行: linguist" ;;
  *)
    echo "注意：$INSTALL_DIR 未在 PATH 中，请手动添加:" >&2
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" >&2 ;;
esac
