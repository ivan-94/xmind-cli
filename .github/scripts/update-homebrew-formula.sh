#!/usr/bin/env bash
set -euo pipefail

DIST_DIR="${DIST_DIR:-target/distrib}"
TAP_REPO="${HOMEBREW_TAP_REPO:-ivan-94/homebrew-tap}"
FORMULA_NAME="${HOMEBREW_FORMULA_NAME:-xmind-cli}"
REPO="${GITHUB_REPOSITORY:-ivan-94/xmind-cli}"
TAG="${GITHUB_REF_NAME:-}"
TOKEN="${HOMEBREW_TAP_TOKEN:-}"
TAP_REMOTE_URL="${HOMEBREW_TAP_REMOTE_URL:-}"

fail() {
  echo "xmind homebrew publish: $*" >&2
  exit 1
}

[ -n "$TAG" ] || fail "GITHUB_REF_NAME is required"
case "$TAG" in
  v*) ;;
  *) fail "expected a v* release tag, got '$TAG'" ;;
esac

[ -f "$DIST_DIR/SHA256SUMS" ] || fail "missing $DIST_DIR/SHA256SUMS"

if [ -z "$TAP_REMOTE_URL" ]; then
  [ -n "$TOKEN" ] || fail "HOMEBREW_TAP_TOKEN is required to push $TAP_REPO"
  TAP_REMOTE_URL="https://x-access-token:${TOKEN}@github.com/${TAP_REPO}.git"
fi

version="${TAG#v}"
class_name="XmindCli"

sha_for() {
  artifact="$1"
  awk -v artifact="$artifact" '$2 == artifact { print $1 }' "$DIST_DIR/SHA256SUMS"
}

artifact_for() {
  target="$1"
  printf 'xmind-cli-%s.tar.gz' "$target"
}

darwin_arm_artifact="$(artifact_for aarch64-apple-darwin)"
darwin_intel_artifact="$(artifact_for x86_64-apple-darwin)"
linux_arm_artifact="$(artifact_for aarch64-unknown-linux-gnu)"
linux_intel_artifact="$(artifact_for x86_64-unknown-linux-gnu)"

darwin_arm_sha="$(sha_for "$darwin_arm_artifact")"
darwin_intel_sha="$(sha_for "$darwin_intel_artifact")"
linux_arm_sha="$(sha_for "$linux_arm_artifact")"
linux_intel_sha="$(sha_for "$linux_intel_artifact")"

for pair in \
  "$darwin_arm_artifact:$darwin_arm_sha" \
  "$darwin_intel_artifact:$darwin_intel_sha" \
  "$linux_arm_artifact:$linux_arm_sha" \
  "$linux_intel_artifact:$linux_intel_sha"; do
  artifact="${pair%%:*}"
  sha="${pair#*:}"
  [ -n "$sha" ] || fail "SHA256SUMS does not contain $artifact"
done

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

git clone "$TAP_REMOTE_URL" "$tmp/tap"
cd "$tmp/tap"
mkdir -p Formula

cat > "Formula/${FORMULA_NAME}.rb" <<FORMULA
class ${class_name} < Formula
  desc "Agent-native CLI for inspecting and editing XMind workbooks"
  homepage "https://github.com/${REPO}"
  version "${version}"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/${REPO}/releases/download/${TAG}/${darwin_arm_artifact}"
      sha256 "${darwin_arm_sha}"
    end

    on_intel do
      url "https://github.com/${REPO}/releases/download/${TAG}/${darwin_intel_artifact}"
      sha256 "${darwin_intel_sha}"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/${REPO}/releases/download/${TAG}/${linux_arm_artifact}"
      sha256 "${linux_arm_sha}"
    end

    on_intel do
      url "https://github.com/${REPO}/releases/download/${TAG}/${linux_intel_artifact}"
      sha256 "${linux_intel_sha}"
    end
  end

  def install
    bin.install "xmind"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/xmind --version")
  end
end
FORMULA

git config user.name "${GITHUB_ACTOR:-github-actions[bot]}"
git config user.email "${GITHUB_ACTOR_ID:-41898282}+${GITHUB_ACTOR:-github-actions[bot]}@users.noreply.github.com"
git add "Formula/${FORMULA_NAME}.rb"

if git diff --cached --quiet; then
  echo "Homebrew formula is already up to date"
  exit 0
fi

git commit -m "${FORMULA_NAME} ${version}"
git push origin HEAD:main
