# xmind patch

## Source Manifest

- Conversation: XMind CLI product design discussion
- GitHub issue #19: patch apply transactional writer
- GitHub issue #23: final documentation synchronization
- Scope: Command reference for declarative batch operations
- Last updated: 2026-05-23

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
    ],
    "backup_path": ".xmind-backups/roadmap-20260523T120000.xmind"
  }
}
```

`patch --apply` plans every operation against an in-memory working copy before
any file write. If a later operation fails, the original workbook is left
untouched and the JSON error includes the failing `operation_index` when known.
Selectors accepted during dry-run must resolve the same way during apply; an
operation cannot report `applied: true` while silently ignoring a selector that
the planner accepted. After planning succeeds, the changed workbook is written
through the transactional writer and must pass structural candidate validation
before replacing the original file.

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
