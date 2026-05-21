# xmind copy

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for copying topics or subtrees
- Last updated: 2026-05-21

## Purpose

Copy a topic and its descendants to another parent.

## Synopsis

```bash
xmind copy <workbook.xmind> --node <selector> --to <selector> (--dry-run | --apply) [--title <new-title>]
```

## Options

- `--node <selector>`: topic or subtree to copy.
- `--to <selector>`: destination parent selector.
- `--title <new-title>`: override copied root title.
- `--position <position>`: destination order.
- `--preserve-ids`: normally false; preserve ids only for diagnostic export flows.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`, `--validate-after`, `--json`.

## Output

```json
{
  "ok": true,
  "command": "copy",
  "result": {
    "copied_root": {
      "source_id": "topic-123",
      "new_id": "topic-456",
      "path": "/Roadmap/Q3/Payment copy"
    }
  }
}
```

## Errors

- `not_found`
- `ambiguous_selector`
- `patch_conflict`
- `write_failed`

## Notes for Agents

Copied topics should receive new ids by default to avoid identity conflicts.
