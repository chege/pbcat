#!/usr/bin/env sh
set -euo pipefail

OWNER="chege"
REPO="pbcat"

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Darwin) target="macos-universal" ;;
  Linux)
    case "$arch" in
      x86_64 | amd64) target="linux-x86_64" ;;
      *) echo "Unsupported Linux architecture: $arch" >&2; exit 1 ;;
    esac
    ;;
  *) echo "Unsupported OS: $os" >&2; exit 1 ;;
esac

install_dir="${INSTALL_DIR:-/usr/local/bin}"

if [ ! -d "$install_dir" ]; then
  echo "Creating install dir $install_dir"
  mkdir -p "$install_dir"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

asset="pbcat-${target}.tar.gz"
url="https://github.com/${OWNER}/${REPO}/releases/latest/download/${asset}"

echo "Downloading $url"
curl -fsSL "$url" -o "$tmp/pbcat.tgz"

echo "Extracting..."
tar -xzf "$tmp/pbcat.tgz" -C "$tmp"

bin_path="$tmp/pbcat-${target}"
if [ ! -x "$bin_path" ]; then
  echo "Binary not found in archive: $bin_path" >&2
  exit 1
fi

echo "Installing to $install_dir/pbcat"
install -m 0755 "$bin_path" "$install_dir/pbcat"

echo "Installed pbcat to $install_dir/pbcat"
