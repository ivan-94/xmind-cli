# xmind delete

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for deleting topics
- Last updated: 2026-05-21

## Purpose

Delete a topic. By default this deletes the selected topic and its descendants as a subtree.

## Synopsis

```bash
xmind delete <workbook.xmind> --node <selector> (--dry-run | --apply) [options]
```

## Options

- `--node <selector>`: topic to delete.
- `--children-only`: delete descendants but keep the topic.
- `--promote-children`: delete the topic but move its children to its parent.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`, `--validate-after`, `--json`.

## Output

```json
{
  "ok": true,
  "command": "delete",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
  "result": {
    "deleted": [
      "/Q2/Old payment",
      "/Q2/Old payment/Risk"
    ]
  }
}
```

## Dry Run Behavior

Dry run must list all paths that would be deleted or promoted.

## Errors

- `not_found`
- `ambiguous_selector`
- `root_operation_not_allowed`
- `validation_failed`
- `write_failed`

## Root Behavior

Deleting `root` or `path:/` is not allowed, including with `--children-only`.
Select a non-root topic when clearing descendants.

## Notes for Agents

Always run `delete` with `--dry-run --json` first unless the target id came from the immediately preceding command.
