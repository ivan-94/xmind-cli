# xmind sheets

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for listing workbook sheets
- Last updated: 2026-05-21

## Purpose

List sheets in workbook order.

## Synopsis

```bash
xmind sheets <workbook.xmind> [--json]
```

## Options

- `--json`: emit sheet objects.
- `--fields id,title,index,topic_count`: choose fields.

## Output

```json
{
  "ok": true,
  "command": "sheets",
  "result": {
    "sheets": [
      { "id": "sheet-1", "index": 0, "title": "Roadmap", "topic_count": 42 }
    ]
  }
}
```

## Errors

- `file_not_found`
- `parse_failed`

## Notes for Agents

Use `--sheet-id` or `--sheet` from this output in later topic commands when multiple sheets exist.

