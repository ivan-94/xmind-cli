# xmind copy

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for copying topics or subtrees
- Last updated: 2026-05-22

## Purpose

Copy a topic and its descendants to another parent.

## Synopsis

```bash
xmind copy [options] --node <selector> --to <selector> (--dry-run | --apply) <workbook.xmind>
```

## Options

- `--node <selector>`: topic or subtree to copy.
- `--to <selector>`: destination parent selector.
- `--title <new-title>`: override copied root title.
- `--position <position>`: destination order.
- `--preserve-ids`: normally false; preserve ids only for diagnostic export flows.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`: create a backup before applying.
- Global output and sheet options are documented in `../global-options.md`.

## Output

```json
{
  "ok": true,
  "command": "copy",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
  "result": {
    "copied_root": {
      "source_id": "topic-123",
      "new_id": "topic-456",
      "path": "/Q3/Payment copy"
    }
  }
}
```

## Errors

- `not_found`
- `ambiguous_selector`
- `root_operation_not_allowed`
- `patch_conflict`
- `write_failed`

## Notes for Agents

Copied topics should receive new ids by default to avoid identity conflicts. Copying `root` or `path:/` is not allowed in the first product contract; export/import should be used for whole-sheet transfer.

The equivalent patch operation is `op: copy` with `preserve_ids: false` by default.
