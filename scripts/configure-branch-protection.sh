#!/usr/bin/env bash
set -euo pipefail

REPO="${XMIND_BRANCH_PROTECTION_REPO:-ivan-94/xmind-cli}"
BRANCH="${XMIND_BRANCH_PROTECTION_BRANCH:-master}"
MODE="${1:-apply}"

usage() {
  cat <<'USAGE'
xmind-cli branch protection setup

Usage:
  scripts/configure-branch-protection.sh [apply|print-json]

Environment:
  XMIND_BRANCH_PROTECTION_REPO     GitHub repository, default ivan-94/xmind-cli.
  XMIND_BRANCH_PROTECTION_BRANCH   Protected branch, default master.

The apply mode requires gh CLI authentication with repository administration
permission. It configures the PRD #1 merge gate: pull requests, strict required
status checks, no force pushes, and no branch deletion.
USAGE
}

protection_json() {
  cat <<'JSON'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Rust quality gate",
      "Stable PR E2E subset",
      "Security"
    ]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 1,
    "require_last_push_approval": false
  },
  "restrictions": null,
  "required_linear_history": false,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "block_creations": false,
  "required_conversation_resolution": false,
  "lock_branch": false,
  "allow_fork_syncing": true
}
JSON
}

case "$MODE" in
  apply)
    command -v gh >/dev/null 2>&1 || {
      echo "missing required command: gh" >&2
      exit 1
    }
    protection_json | gh api \
      --method PUT \
      -H "Accept: application/vnd.github+json" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      "/repos/${REPO}/branches/${BRANCH}/protection" \
      --input -
    ;;
  print-json)
    protection_json
    ;;
  -h|--help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
