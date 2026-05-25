# Examples

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Source manifest for machine-readable example inputs and outputs
- Last updated: 2026-05-22

## Notes

JSON example files in this directory intentionally do not include `Source Manifest` fields inside the JSON object. They are meant to represent real CLI input or output shapes that agents can parse directly.

This README carries the documentation source manifest for the example set.

## Fixture Commands

Ordinary `bash` blocks are illustrative and are not executed automatically:

```bash
xmind inspect tests/fixtures/xmind/minimal.xmind --json
xmind sheets tests/fixtures/xmind/minimal.xmind --json
xmind tree tests/fixtures/xmind/minimal.xmind --depth 2 --json
xmind get tests/fixtures/xmind/minimal.xmind --node path:/Q2/Payment --json
xmind find tests/fixtures/xmind/minimal.xmind --title Payment --json
xmind add-tree tests/fixtures/xmind/minimal.xmind --parent path:/Q2 --input docs/examples/simple-tree.yaml --dry-run --json
xmind patch tests/fixtures/xmind/minimal.xmind --ops docs/examples/patch-add-tree.yaml --dry-run --json
```

Blocks marked `bash e2e` are executable documentation examples. They are
checked by `tests/cli/doc_examples_test.rs`; use committed fixtures for
read-only commands and copy fixtures to temporary files before any `--apply`
command.

```bash e2e
xmind tree tests/fixtures/xmind/minimal.xmind --json
```

```bash e2e
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
cp tests/fixtures/xmind/minimal.xmind "$tmp_dir/roadmap.xmind"
xmind add "$tmp_dir/roadmap.xmind" --parent path:/Q2 --title Refund --apply --json
xmind validate "$tmp_dir/roadmap.xmind" --json
```
