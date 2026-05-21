# xmind tree

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for displaying topic trees
- Last updated: 2026-05-21

## Purpose

Display a sheet tree or a subtree.

## Synopsis

```bash
xmind tree <workbook.xmind> [--sheet <title>] [--node <selector>] [--depth <n>] [--json]
```

## Options

- `--sheet`, `--sheet-id`, `--sheet-index`: select sheet.
- `--node <selector>`: root of subtree to display.
- `--depth <n>`: maximum descendant depth.
- `--fields <fields>`: include selected fields.
- `--include-assets`: include topic image and asset references.
- `--json`: emit structured tree.

## Output

```json
{
  "ok": true,
  "command": "tree",
  "result": {
    "sheet": "Roadmap",
    "root": {
      "id": "topic-root",
      "path": "/Roadmap",
      "title": "Roadmap",
      "children": []
    }
  }
}
```

## Errors

- `ambiguous_sheet`
- `sheet_not_found`
- `not_found`
- `ambiguous_selector`

## Notes for Agents

Use `--depth` aggressively to keep context small. Use paths or ids from this output for write commands.
