# xmind set

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for updating topic fields
- Last updated: 2026-05-21

## Purpose

Update fields on one topic.

## Synopsis

```bash
xmind set <workbook.xmind> --node <selector> (--dry-run | --apply) [field options]
```

## Options

- `--node <selector>`: topic to update.
- `--title <title>`: replace title.
- `--note <text>`: replace note.
- `--append-note <text>`: append to note.
- `--set-labels <csv>`: replace all labels.
- `--add-label <label>`: add one label.
- `--remove-label <label>`: remove one label.
- `--set-markers <csv>`: replace all markers.
- `--add-marker <marker>`: add one marker.
- `--remove-marker <marker>`: remove one marker.
- `--hyperlink <url>`: replace hyperlink.
- `--image <file>`: attach or replace topic image.
- `--image-alt <text>`: alt text for topic image.
- `--clear labels|markers|note|hyperlink|image`: clear fields.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`, `--validate-after`, `--json`.

## Output

```json
{
  "ok": true,
  "command": "set",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
  "result": {
    "updated": {
      "id": "topic-123",
      "path": "/Q2/Payments",
      "changed_fields": ["title", "note"]
    }
  }
}
```

## Errors

- `not_found`
- `ambiguous_selector`
- `invalid_usage`
- `unsupported_asset_type`
- `root_operation_not_allowed`
- `write_failed`

## Root Behavior

`set --node root` and `set --node path:/` may update root fields only when that field is editable in `compatibility-matrix.md`. Unsupported root edits fail with `root_operation_not_allowed`.

## Clear Syntax

`--clear` accepts one field per flag and may be repeated:

```bash
xmind set roadmap.xmind --node "id:topic-123" --clear labels --clear markers --apply
```

Comma-separated values are invalid. Use repeated flags to avoid shell parsing ambiguity.

## Notes for Agents

When renaming a topic, prefer id selectors because path selectors may become invalid after the change.
