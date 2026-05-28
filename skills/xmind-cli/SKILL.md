---
name: xmind-cli
description: Use this skill when working with .xmind workbooks through the xmind command line tool, especially for inspecting workbook structure, finding topics, reading topic trees, previewing safe edits, applying explicit mutations with backups, validating workbooks, importing or exporting structured content, or debugging real XMind package compatibility.
---

# xmind-cli

`xmind-cli` turns XMind workbooks into inspectable, queryable, editable
structured data. Use this skill to operate through the `xmind` CLI instead of
editing the `.xmind` zip package directly.

## Quick start

Prefer JSON for agent work:

```bash
xmind inspect plan.xmind --json
xmind sheets plan.xmind --json
xmind tree plan.xmind --sheet "Roadmap" --depth 3 --json
xmind find plan.xmind --title "Payment" --json
```

If `xmind` is not on `PATH` inside this repository, use:

```bash
cargo run -- tree tests/fixtures/xmind/minimal.xmind --depth 2 --json
```

For a built release binary, `target/release/xmind` is also valid.

## Choose the command

| User intent | First command | Follow-up |
| --- | --- | --- |
| Understand a workbook | `inspect`, `sheets`, `tree` | `validate` if integrity matters |
| Locate a topic | `find` | `get` with a stable selector |
| Read one topic or subtree | `get`, `tree` | Add `--fields` or compact output for large maps |
| Add one topic | `add --dry-run --json` | Repeat with `--apply --backup --json` |
| Add a subtree | `add-tree --dry-run --json` | Use YAML/JSON tree input |
| Update fields | `set --dry-run --json` | Update only explicit fields |
| Remove content | `delete --dry-run --json` | Check descendants before apply |
| Reorganize content | `move` or `copy` with `--dry-run --json` | Prefer `id:` selectors |
| Multiple changes | `patch --ops <file> --dry-run --json` | Apply once after reviewing the plan |
| Externalize content | `export` | Choose `--format markdown`, `json`, `outline`, or `text` |
| Create/update from content | `import --dry-run --json` | Treat as a mutating command |
| Recover | `backup`, `restore` | Validate after restore |

## Read workflow

1. Start with `inspect` to learn workbook format, sheet count, and capabilities.
2. Use `sheets` before assuming sheet names or indices.
3. Use `tree --depth` to keep output bounded.
4. Use `find` for human terms, then `get` for the resolved topic.
5. Use `--format compact-json --fields id,path,title` when the map is large.

Example:

```bash
xmind inspect plan.xmind --json
xmind sheets plan.xmind --json
xmind tree plan.xmind --sheet "Roadmap" --depth 2 --json --format compact-json --fields id,path,title
xmind find plan.xmind --sheet "Roadmap" --title "Payment" --json
xmind get plan.xmind --node "id:topic-123" --json
```

## Edit workflow

Every workbook mutation uses the same safety loop:

1. Resolve the target with read-only commands.
2. Prefer an `id:` selector from JSON output for the final write target.
3. Run the exact mutation once with `--dry-run --json`.
4. Inspect the planned diff and result summary.
5. Run the same command with `--apply --backup --json` only when the dry run is
   correct.
6. Run `xmind validate <file> --json`.

Never report a write as complete unless the JSON result includes
`applied: true`.

Example:

```bash
xmind add-tree plan.xmind \
  --parent "id:topic-123" \
  --input generated-tree.yaml \
  --dry-run \
  --json

xmind add-tree plan.xmind \
  --parent "id:topic-123" \
  --input generated-tree.yaml \
  --apply \
  --backup \
  --json

xmind validate plan.xmind --json
```

## Selectors

Use selectors deliberately:

- `id:<topic-id>`: best for mutations after discovery.
- `path:/A/B`: good for human-readable targets when titles are unique.
- title search: good for discovery, risky for final writes when duplicates may
  exist.

If a selector returns `ambiguous_selector`, read `error.candidates`, choose a
candidate id, and retry. Do not pick a candidate by position unless the user
explicitly asked for that topic.

## Structured input

Use tree input for generated subtrees instead of repeated one-topic commands.
Use patch input for multi-step edits that should be reviewed as one plan.

Read [references/examples.md](references/examples.md) before generating YAML,
JSON, Markdown outline, or patch payloads.

## Error handling

When `--json` is used, recover from the structured error envelope:

- `ambiguous_selector`: choose a candidate `id:` and retry.
- `not_found`: rediscover with `tree` or `find`.
- `invalid_tree_input`: fix the input at `field_path`.
- `invalid_patch`: fix the operation at `operation_index`.
- `patch_conflict`: recompute against the current tree.
- `validation_failed`: stop, inspect diagnostics, and do not retry blindly.
- `root_operation_not_allowed`: target a child topic or use an allowed root edit.

## Real workbook debugging

For real user files, first prove CLI behavior:

```bash
xmind inspect real.xmind --json
xmind validate real.xmind --json
xmind tree real.xmind --depth 2 --json
```

Only inspect zip internals after capturing CLI output, and keep that inspection
read-only unless the user explicitly asked for low-level repair. See
[references/package-debugging.md](references/package-debugging.md).

## References

- [references/workflows.md](references/workflows.md): task workflows and
  acceptance checklist.
- [references/contracts.md](references/contracts.md): command contracts, JSON
  envelopes, selectors, mutation rules, and doc map.
- [references/examples.md](references/examples.md): reusable YAML, JSON,
  Markdown outline, and patch payload examples.
- [references/package-debugging.md](references/package-debugging.md): real
  `.xmind` package diagnosis and preservation rules.
