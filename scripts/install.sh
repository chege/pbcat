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
release_ref="${RELEASE_TAG:-latest}"

if [ "$release_ref" = "latest" ]; then
  api_url="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"
else
  api_url="https://api.github.com/repos/${OWNER}/${REPO}/releases/tags/${release_ref}"
fi

echo "Resolving asset for ${release_ref} (${asset})..."
asset_url="$(
  curl -fsSL "$api_url" \
  | awk -F'"' -v pat="$asset" '/browser_download_url/ { if ($4 ~ pat) { print $4 } }' \
  | head -n1
)"

if [ -z "$asset_url" ]; then
  echo "Could not find asset $asset at $api_url" >&2
  echo "You can install from source instead: cargo install --git https://github.com/${OWNER}/${REPO}.git --locked" >&2
  exit 1
fi

echo "Downloading $asset_url"
if ! curl -fL "$asset_url" -o "$tmp/pbcat.tgz"; then
  echo "Download failed. Install from source: cargo install --git https://github.com/${OWNER}/${REPO}.git --locked" >&2
  exit 1
fi

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
