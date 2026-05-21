# xmind add

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for adding one topic
- Last updated: 2026-05-21

## Purpose

Add one topic under a parent.

## Synopsis

```bash
xmind add <workbook.xmind> --parent <selector> --title <title> (--dry-run | --apply) [options]
```

## Options

- `--parent <selector>`: parent topic.
- `--title <title>`: new topic title.
- `--note <text>`: optional note.
- `--label <label>`: repeatable label.
- `--marker <marker>`: repeatable marker.
- `--hyperlink <url>`: optional hyperlink.
- `--image <file>`: attach a topic image from a local file.
- `--image-alt <text>`: alt text for the topic image.
- `--position <position>`: `first`, `last`, `index:N`, `before:<selector>`, or `after:<selector>`.
- `--create-missing-path`: create missing path parent segments.
- `--if-exists error|skip|merge|replace|rename`: duplicate handling.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`, `--validate-after`, `--json`.

## Output

```json
{
  "ok": true,
  "command": "add",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
  "result": {
    "created": {
      "id": "topic-new",
      "path": "/Q2/Payment"
    }
  }
}
```

## Dry Run Behavior

Dry run returns the would-be parent, position, created path, and diff.

## Errors

- `not_found`
- `ambiguous_selector`
- `invalid_usage`
- `unsupported_asset_type`
- `write_failed`

## Notes for Agents

Use `add-tree` instead when creating more than one related topic.
