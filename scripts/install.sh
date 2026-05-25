#!/usr/bin/env bash
set -euo pipefail

REPO="${XMIND_INSTALL_REPO:-ivan-94/xmind-cli}"
VERSION="${XMIND_INSTALL_VERSION:-latest}"
INSTALL_DIR="${XMIND_INSTALL_DIR:-$HOME/.local/bin}"
DRY_RUN=0
BASE_URL="${XMIND_INSTALL_BASE_URL:-}"

usage() {
  cat <<'USAGE'
xmind-cli install script

Usage:
  scripts/install.sh [--dry-run] [--version <tag>] [--install-dir <dir>] [--repo <owner/repo>]

Options:
  --dry-run          Preview platform, artifact, checksum, and install path without writing files.
  --version <tag>    Release tag to install, for example v0.1.0. Defaults to latest.
  --install-dir DIR  Directory for the xmind binary. Defaults to ~/.local/bin.
  --repo OWNER/REPO  GitHub repository. Defaults to ivan-94/xmind-cli.
  -h, --help         Show this help.

Environment for tests or mirrors:
  XMIND_INSTALL_BASE_URL   Directory or URL containing the release assets and SHA256SUMS.
  XMIND_INSTALL_OS         Override uname -s.
  XMIND_INSTALL_ARCH       Override uname -m.
USAGE
}

fail() {
  echo "xmind install: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command '$1'. Install it and rerun, or use: cargo install --locked --git https://github.com/$REPO"
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run|--preview)
      DRY_RUN=1
      shift
      ;;
    --version)
      [ "$#" -ge 2 ] || fail "--version requires a release tag, for example v0.1.0"
      VERSION="$2"
      shift 2
      ;;
    --install-dir)
      [ "$#" -ge 2 ] || fail "--install-dir requires a directory"
      INSTALL_DIR="$2"
      shift 2
      ;;
    --repo)
      [ "$#" -ge 2 ] || fail "--repo requires OWNER/REPO"
      REPO="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument '$1'. Run with --help for usage."
      ;;
  esac
done

os="${XMIND_INSTALL_OS:-$(uname -s)}"
arch="${XMIND_INSTALL_ARCH:-$(uname -m)}"
archive_ext="tar.gz"
binary_name="xmind"
case "$os:$arch" in
  Darwin:arm64|Darwin:aarch64)
    target="aarch64-apple-darwin"
    ;;
  Darwin:x86_64|Darwin:amd64)
    target="x86_64-apple-darwin"
    ;;
  Linux:x86_64|Linux:amd64)
    target="x86_64-unknown-linux-gnu"
    ;;
  Linux:aarch64|Linux:arm64)
    target="aarch64-unknown-linux-gnu"
    ;;
  MINGW*:x86_64|MSYS*:x86_64|CYGWIN*:x86_64|Windows_NT:x86_64|Windows:x86_64|Windows:amd64)
    target="x86_64-pc-windows-msvc"
    archive_ext="zip"
    binary_name="xmind.exe"
    ;;
  *)
    fail "unsupported platform '$os/$arch'. Supported release targets are macOS Apple Silicon, macOS Intel, Linux x86_64 GNU, Linux arm64 GNU, and Windows x86_64 MSVC. For this platform use: cargo install --locked --git https://github.com/$REPO"
    ;;
esac

if [ "$VERSION" = "latest" ] && [ -z "$BASE_URL" ]; then
  need_cmd curl
  VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
  [ -n "$VERSION" ] || fail "could not resolve the latest release tag for $REPO. Pass --version vX.Y.Z explicitly."
fi

artifact="xmind-cli-${VERSION}-${target}.${archive_ext}"
if [ -n "$BASE_URL" ]; then
  base="${BASE_URL%/}"
else
  base="https://github.com/$REPO/releases/download/$VERSION"
fi
artifact_url="$base/$artifact"
checksums_url="$base/SHA256SUMS"
install_path="$INSTALL_DIR/$binary_name"

cat <<PLAN
xmind install plan:
  release: $VERSION
  platform: $target
  artifact: $artifact
  artifact_url: $artifact_url
  checksums_url: $checksums_url
  install_path: $install_path
PLAN

if [ "$DRY_RUN" -eq 1 ]; then
  echo "DRY RUN: no files were downloaded, extracted, or installed."
  exit 0
fi

need_cmd shasum
case "$archive_ext" in
  tar.gz) need_cmd tar ;;
  zip) need_cmd unzip ;;
esac
if [[ "$artifact_url" == http://* || "$artifact_url" == https://* ]]; then
  need_cmd curl
fi

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

download() {
  src="$1"
  dest="$2"
  if [[ "$src" == http://* || "$src" == https://* ]]; then
    curl -fL --proto '=https' --tlsv1.2 -o "$dest" "$src"
  elif [[ "$src" == file://* ]]; then
    cp "${src#file://}" "$dest"
  else
    cp "$src" "$dest"
  fi
}

archive_path="$tmp/$artifact"
checksums_path="$tmp/SHA256SUMS"
checksum_line="$tmp/$artifact.sha256"

download "$artifact_url" "$archive_path" || fail "failed to download $artifact_url. Check the release tag and platform artifact name."
download "$checksums_url" "$checksums_path" || fail "failed to download $checksums_url. Refusing to install without checksums."

awk -v artifact="$artifact" '$2 == artifact { print $1 "  " artifact }' "$checksums_path" > "$checksum_line"
if [ ! -s "$checksum_line" ]; then
  fail "SHA256SUMS does not contain an entry for $artifact. Refusing to guess or install a different artifact."
fi

(
  cd "$tmp"
  shasum -a 256 -c "$(basename "$checksum_line")" >/dev/null
) || fail "checksum verification failed for $artifact. delete the downloaded file, download it again from the GitHub Release, and do not run the binary until verification passes."

extract_dir="$tmp/extract"
mkdir -p "$extract_dir"
case "$archive_ext" in
  tar.gz)
    tar -xzf "$archive_path" -C "$extract_dir"
    ;;
  zip)
    unzip -q "$archive_path" -d "$extract_dir"
    ;;
esac

found_binary="$(find "$extract_dir" -type f -name "$binary_name" -perm -u+x | head -n 1)"
if [ -z "$found_binary" ]; then
  found_binary="$(find "$extract_dir" -type f -name "$binary_name" | head -n 1)"
fi
[ -n "$found_binary" ] || fail "archive $artifact did not contain $binary_name. Refusing to install an unexpected artifact."

mkdir -p "$INSTALL_DIR"
cp "$found_binary" "$install_path"
chmod 0755 "$install_path"

echo "Installed $binary_name to $install_path"
echo "Verify with: $install_path --version"
