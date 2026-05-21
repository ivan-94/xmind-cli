# xmind get

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for reading one topic
- Last updated: 2026-05-21

## Purpose

Return one topic and optionally its descendants.

## Synopsis

```bash
xmind get <workbook.xmind> --node <selector> [--depth <n>] [--json]
```

## Options

- `--node <selector>`: required topic selector.
- `--depth <n>`: include descendants up to depth.
- `--fields <fields>`: choose topic fields.
- `--include-assets`: include topic image and asset references.
- `--json`: emit structured topic.

## Output

```json
{
  "ok": true,
  "command": "get",
  "result": {
    "topic": {
      "id": "topic-123",
      "path": "/Roadmap/Q2/Payment",
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

Use `get` after `find` when you need full details for one candidate.
