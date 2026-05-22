# xmind tree

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for displaying topic trees
- Last updated: 2026-05-22

## Purpose

Display a sheet tree or a subtree.

## Synopsis

```bash
xmind tree [options] <workbook.xmind>
```

## Options

- `--depth <n>`: maximum descendant depth.
- `--include-assets`: include topic image and asset references.
- Global output and sheet options are documented in `../global-options.md`.

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
