# CLI Overview

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command inventory and top-level CLI shape
- Last updated: 2026-05-21

## Synopsis

```bash
xmind <command> <workbook.xmind> [options]
```

## Command Groups

### Inspection

- `inspect`: summarize workbook format, sheets, and capabilities.
- `sheets`: list sheets.
- `tree`: show a sheet or subtree.
- `find`: search topics.
- `get`: return one topic.

### Editing

- `add`: add one topic.
- `add-tree`: add a subtree.
- `set`: update topic fields.
- `delete`: delete a topic or subtree.
- `move`: move a topic or subtree.
- `copy`: copy a topic or subtree.

### Batch and Exchange

- `patch`: apply declarative operations.
- `diff`: compare workbooks or preview operation diffs.
- `validate`: validate workbook integrity.
- `export`: export to JSON, Markdown, outline, or text.
- `import`: create or update from external structured input.

### Recovery

- `backup`: create a backup.
- `restore`: restore a backup.

### Shell Integration

- `completion`: generate shell completion scripts.

## Common Read Example

```bash
xmind tree roadmap.xmind --sheet "Roadmap" --depth 3 --json
```

## Common Write Example

```bash
xmind add-tree roadmap.xmind \
  --parent "path:/Q2" \
  --input payment.yaml \
  --dry-run \
  --json
```

## Agent Contract

Every command should support `--json`. Every workbook-mutating command must receive exactly one of `--dry-run` or `--apply`, and write commands should support `--backup` where in-place edits are possible. Run `xmind validate` explicitly when an acceptance flow needs a separate validation result.

Detailed mutation behavior is defined in `mutation-semantics.md`. Patch operation behavior is defined in `patch-operations.md`.
