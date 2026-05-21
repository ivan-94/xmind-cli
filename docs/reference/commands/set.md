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
  "result": {
    "updated": {
      "id": "topic-123",
      "path": "/Roadmap/Q2/Payments",
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
- `write_failed`

## Notes for Agents

When renaming a topic, prefer id selectors because path selectors may become invalid after the change.
