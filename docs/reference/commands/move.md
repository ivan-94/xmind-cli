# xmind move

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for moving topics or subtrees
- Last updated: 2026-05-22

## Purpose

Move a topic and its descendants to another parent or sibling position.

## Synopsis

```bash
xmind move [options] --node <selector> --to <selector> (--dry-run | --apply) <workbook.xmind>
```

## Options

- `--node <selector>`: topic or subtree to move.
- `--to <selector>`: destination parent selector.
- `--position <position>`: destination order.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`: create a backup before applying.
- Global output and sheet options are documented in `../global-options.md`.

## Output

```json
{
  "ok": true,
  "command": "move",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
  "result": {
    "moved": {
      "id": "topic-123",
      "from_path": "/Q2/Payment",
      "to_path": "/Q3/Payment"
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

The command must reject moves that would make a topic a descendant of itself. Moving `root` or `path:/` is not allowed.
