# xmind get

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for reading one topic
- Last updated: 2026-05-22

## Purpose

Return one topic and optionally its descendants.

## Synopsis

```bash
xmind get [options] --node <selector> <workbook.xmind>
```

## Options

- `--node <selector>`: required topic selector.
- `--depth <n>`: include descendants up to depth.
- `--include-assets`: include topic image and asset references.
- Global output and sheet options are documented in `../global-options.md`.

## Output

```json
{
  "ok": true,
  "command": "get",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "topic": {
      "id": "topic-123",
      "path": "/Q2/Payment",
      "title": "Payment",
      "note": "",
      "labels": [],
      "markers": [],
      "image": null,
      "children_count": 3
    }
  }
}
```

## Errors

- `not_found`
- `ambiguous_selector`

## Notes for Agents

Use `get` after `find` when you need full details for one candidate. `get` returns `result.topic`; `tree` returns `result.root`. Both use the same topic field names, but `tree` is optimized for hierarchy and `get` is optimized for one topic's detailed fields.

Legal `--fields` values are documented in `../fields.md`.
