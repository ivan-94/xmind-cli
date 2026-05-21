# xmind find

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for searching topics
- Last updated: 2026-05-21

## Purpose

Search topics by title, note, labels, markers, path, hyperlink, depth, or query expression.

## Synopsis

```bash
xmind find <workbook.xmind> [--sheet <title>] [--title <text>] [--query <expr>] [--json]
```

## Options

- `--title <text>`: exact or default title search.
- `--contains <text>`: title or note contains text.
- `--query <expr>`: query selector expression.
- `--limit <n>`: maximum matches.
- `--fields <fields>`: choose result fields.
- `--json`: emit structured matches.

## Output

```json
{
  "ok": true,
  "command": "find",
  "result": {
    "matches": [
      {
        "id": "topic-123",
        "path": "/Roadmap/Q2/Payment",
        "title": "Payment",
        "sheet": "Roadmap",
        "children_count": 3
      }
    ]
  }
}
```

## Errors

- `invalid_usage`
- `ambiguous_sheet`
- `parse_failed`

## Notes for Agents

`find` can return multiple matches. Do not feed `title:` selectors into write commands until you know they are unique.

