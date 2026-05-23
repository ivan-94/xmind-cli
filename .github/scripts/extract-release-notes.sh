#!/usr/bin/env bash
set -euo pipefail

tag="${1:?usage: extract-release-notes.sh <tag> [changelog] [output]}"
changelog="${2:-CHANGELOG.md}"
output="${3:-target/release-notes.md}"
tmp="${output}.tmp"

mkdir -p "$(dirname "$output")"

if ! awk -v tag="$tag" '
  BEGIN {
    in_section = 0
    found = 0
  }
  /^##[[:space:]]+/ {
    if (in_section) {
      exit
    }
    heading = $0
    sub(/^##[[:space:]]+/, "", heading)
    if (heading == tag || index(heading, tag " - ") == 1 || index(heading, tag " (") == 1) {
      in_section = 1
      found = 1
    }
  }
  in_section {
    print
  }
  END {
    if (!found) {
      exit 42
    }
  }
' "$changelog" > "$tmp"; then
  rm -f "$tmp"
  echo "CHANGELOG.md is missing a release notes section for ${tag}" >&2
  echo "Expected a heading like: ## ${tag} - YYYY-MM-DD" >&2
  exit 1
fi

mv "$tmp" "$output"
echo "release_notes_path=$output"
