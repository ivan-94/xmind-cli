# xmind inspect

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for workbook inspection
- Last updated: 2026-05-21

## Purpose

Summarize workbook structure, detected format, sheet count, and high-level capabilities without printing the full topic tree.

## Synopsis

```bash
xmind inspect <workbook.xmind> [--json]
```

## When to Use

Use first when an agent receives an unknown XMind file.

## Arguments

- `<workbook.xmind>`: workbook to inspect.

## Options

- `--json`: emit structured output.
- `--fields`: limit fields.

## Output

```json
{
  "ok": true,
  "command": "inspect",
  "result": {
    "format": "xmind-zen",
    "sheet_count": 2,
    "sheets": [
      { "id": "sheet-1", "title": "Roadmap", "topic_count": 42 }
    ],
    "capabilities": {
      "can_read_topics": true,
      "can_preserve_unknown": true
    }
  }
}
```

## Errors

- `file_not_found`
- `parse_failed`

## Notes for Agents

Prefer `inspect` before assuming there is only one sheet.

