# xmind export

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for exporting workbook content
- Last updated: 2026-05-21

## Purpose

Export a workbook, sheet, or subtree to a portable representation.

## Synopsis

```bash
xmind export <workbook.xmind> --format json|markdown|outline|text|assets [options]
```

## Options

- `--format json|markdown|outline|text|assets`: export payload format.
- `--sheet <title>`: selected sheet.
- `--node <selector>`: selected subtree.
- `--depth <n>`: limit descendants.
- `--output <file>`: write to file.
- `--include-notes`: include notes.
- `--include-metadata`: include labels, markers, hyperlinks.
- `--include-assets`: include topic image references in structured exports.

## Output

If `--output` is omitted, export writes to stdout.

## Errors

- `not_found`
- `ambiguous_selector`
- `unsupported_asset_type`
- `write_failed`

## Notes for Agents

Use Markdown export for human review and JSON export for later automated patch generation. Use `--format assets --output <dir>` to export workbook image resources.
