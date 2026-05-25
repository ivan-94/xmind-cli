#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
hat_workspace="$repo_root/.scratch/hat/prd-1-github-release-e2e"
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
seed_records=fixtures:committed-xmind-fixtures
cleanup=$repo_root/hats/20260523-prd-1-github-release-e2e/prepare.sh cleanup
guide=$repo_root/hats/20260523-prd-1-github-release-e2e/guide.md
END_HAT_PREPARE_SUMMARY
SUMMARY
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

prepare() {
  require_cmd cargo
  require_cmd git
  require_cmd bash

  mkdir -p "$hat_workspace"

  if [ ! -f "$repo_root/tests/fixtures/xmind/minimal.xmind" ]; then
    echo "missing required fixture: tests/fixtures/xmind/minimal.xmind" >&2
    exit 1
  fi

  (
    cd "$repo_root"
    cargo build --workspace
    ./target/debug/xmind --help > "$hat_workspace/xmind-help.txt"
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
