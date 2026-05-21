# xmind patch

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for declarative batch operations
- Last updated: 2026-05-21

## Purpose

Apply a declarative operation file to a workbook.

## Synopsis

```bash
xmind patch <workbook.xmind> --ops <ops.yaml|ops.json> --dry-run
xmind patch <workbook.xmind> --ops <ops.yaml|ops.json> --apply --backup
```

## Options

- `--ops <file>`: patch file.
- `--dry-run`: validate and preview.
- `--apply`: write changes.
- `--atomic`: apply all or none; preferred default.
- `--backup`, `--backup-dir`, `--validate-after`, `--json`.

## Supported Operations

- `assert_exists`
- `assert_not_exists`
- `add`
- `add_tree`
- `set`
- `replace_tree`
- `merge_tree`
- `delete`
- `delete_tree`
- `move`
- `move_tree`
- `copy`
- `clone_tree`
- `ensure_path`
- `sort_children`
- `set_tree_metadata`

Detailed operation semantics are defined in `../patch-operations.md`.

## Output

```json
{
  "ok": true,
  "command": "patch",
  "result": {
    "summary": {
      "added": 8,
      "updated": 1,
      "deleted": 0,
      "moved": 0
    },
    "operations": [
      { "index": 0, "op": "add_tree", "status": "applied" }
    ]
  }
}
```

## Errors

- `invalid_patch`
- `invalid_tree_input`
- `not_found`
- `ambiguous_selector`
- `patch_conflict`
- `validation_failed`
- `write_failed`

## Notes for Agents

This is the primary interface for nontrivial automated edits. Generate a patch, run dry-run, inspect JSON, then apply with backup.
