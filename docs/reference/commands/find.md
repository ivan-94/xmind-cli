# xmind find

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for searching topics
- Last updated: 2026-05-22

## Purpose

Search topics by title, note, labels, markers, path, hyperlink, depth, or query expression.

## Synopsis

```bash
xmind find [options] <workbook.xmind>
```

## Options

- `--title <text>`: exact title match.
- `--title-contains <text>`: substring title match.
- `--contains <text>`: case-sensitive substring search across title and note.
- `--query <expr>`: query selector expression.
- `--limit <n>`: maximum matches.
- `--offset <n>`: skip matches before returning results.
- Global output and sheet options are documented in `../global-options.md`.

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
