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

- `--title <text>`: exact title match.
- `--title-contains <text>`: substring title match.
- `--contains <text>`: case-sensitive substring search across title and note.
- `--query <expr>`: query selector expression.
- `--limit <n>`: maximum matches.
- `--fields <fields>`: choose result fields.
- `--format compact-json`: compact the `result` payload when used with `--json`.
- `--json`: emit structured matches.

## Output

```json
{
  "ok": true,
  "command": "find",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "matches": [
      {
        "id": "topic-123",
        "path": "/Q2/Payment",
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

`find` can return multiple matches. `--title` is exact and case-sensitive unless a future option says otherwise. Use `--title-contains` for substring search. Do not feed `title:` selectors into write commands until you know they are unique.

Legal `--fields` values are documented in `../fields.md`.
