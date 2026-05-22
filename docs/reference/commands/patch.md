# xmind patch

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for declarative batch operations
- Last updated: 2026-05-22

## Purpose

Apply a declarative operation file to a workbook.

## Synopsis

```bash
xmind patch [options] --ops <ops.yaml|ops.json> --dry-run <workbook.xmind>
xmind patch [options] --ops <ops.yaml|ops.json> --apply --backup <workbook.xmind>
```

## Options

- `--ops <file>`: patch file.
- `--dry-run`: validate and preview.
- `--apply`: write changes.
- `--backup`: create a backup before applying.
- Global output and sheet options are documented in `../global-options.md`.

## Supported Operations

- `assert_exists`
- `assert_not_exists`
- `add`
- `add_tree`
- `set`
- `replace_tree`
- `merge_tree`
- `delete`
- `move`
- `copy`
- `ensure_path`
- `sort_children`
- `set_tree_metadata`

Accepted aliases: `delete_tree`, `move_tree`, and `clone_tree`. Agents should generate canonical names only. Detailed operation semantics are defined in `../patch-operations.md`.

## Output

```json
{
  "ok": true,
  "command": "patch",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
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
- `root_operation_not_allowed`
- `validation_failed`
- `write_failed`

## Notes for Agents

This is the primary interface for nontrivial automated edits. Generate a patch, run dry-run, inspect JSON, then apply with backup.

Patch operations follow the root topic rules in `../mutation-semantics.md`. In particular, `delete_tree`, `move_tree`, and `replace_tree` reject root targets.
