# xmind sheets

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for listing workbook sheets
- Last updated: 2026-05-22

## Purpose

List sheets in workbook order.

## Synopsis

```bash
xmind sheets [options] <workbook.xmind>
```

## Options

- Global output and sheet options are documented in `../global-options.md`.

## Output

```json
{
  "ok": true,
  "command": "sheets",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
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

Legal `--fields` values are documented in `../fields.md`.
