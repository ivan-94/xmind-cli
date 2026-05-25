#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
hat_workspace="$repo_root/.scratch/hat/prd-26-30-readme-help-polish"
command_name="${1:-info}"

print_summary() {
  local status="$1"
  cat <<SUMMARY
HAT_PREPARE_SUMMARY
mode=blank
status=$status
app_url=n/a
database=n/a
schema_version=n/a
seed_records=docs-and-cli-help-artifacts
cleanup=$repo_root/hats/20260525-prd-26-30-readme-help-polish/prepare.sh cleanup
guide=$repo_root/hats/20260525-prd-26-30-readme-help-polish/guide.md
END_HAT_PREPARE_SUMMARY
SUMMARY
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

check_required_files() {
  local required_files=(
    "README.md"
    "README.zh-CN.md"
    "AGENTS.md"
    "docs/README.md"
    "docs/reference/cli-overview.md"
    "docs/technical/README.md"
    "docs/examples/README.md"
    "docs/prd/26-30/implementation-notes.html"
  )

  for file in "${required_files[@]}"; do
    if [ ! -f "$repo_root/$file" ]; then
      echo "missing required file: $file" >&2
      exit 1
    fi
  done
}

prepare() {
  require_cmd cargo
  require_cmd git
  require_cmd bash
  check_required_files

  mkdir -p "$hat_workspace"

  (
    cd "$repo_root"
    cargo build --workspace
    ./target/debug/xmind --help > "$hat_workspace/xmind-help.txt"
    ./target/debug/xmind --json > "$hat_workspace/xmind-empty-json-help.txt"
    ./target/debug/xmind set --help > "$hat_workspace/xmind-set-help.txt"
    ./target/debug/xmind diff --help > "$hat_workspace/xmind-diff-help.txt"
    ./target/debug/xmind --version > "$hat_workspace/xmind-version.txt"
  )

  print_summary "prepared"
}

cleanup() {
  rm -rf "$hat_workspace"
  print_summary "cleaned"
}

info() {
  print_summary "not-run"
}

case "$command_name" in
  prepare)
    prepare
    ;;
  cleanup)
    cleanup
    ;;
  info)
    info
    ;;
  *)
    echo "usage: $0 [info|prepare|cleanup]" >&2
    exit 2
    ;;
esac
