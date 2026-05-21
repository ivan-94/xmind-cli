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
- `--format compact-json`: compact the `result` payload when used with `--json`.
- `--json`: emit structured tree.

## Output

```json
{
  "ok": true,
  "command": "tree",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "sheet": "Roadmap",
    "root": {
      "id": "topic-root",
      "path": "/",
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

Use `tree` when hierarchy is the primary information you need. Use `--depth`, `--fields`, and `--format compact-json` aggressively to keep context small. Use paths or ids from this output for write commands.

Legal `--fields` values are documented in `../fields.md`.
