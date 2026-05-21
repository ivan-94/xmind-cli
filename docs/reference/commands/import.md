# xmind import

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for importing structured content
- Last updated: 2026-05-21

## Purpose

Create a new workbook or update an existing workbook from Markdown, YAML, or JSON tree input.

## Synopsis

```bash
xmind import --input outline.md --output roadmap.xmind (--dry-run | --apply)
xmind import --input tree.yaml --into roadmap.xmind --parent "path:/Roadmap/Q2" (--dry-run | --apply)
```

## Options

- `--input <file>`: input file.
- `--format markdown|yaml|json|auto`: input format.
- `--markdown-mode heading|list|hybrid|auto`: Markdown parsing mode.
- `--output <file>`: create output workbook.
- `--into <workbook.xmind>`: import into existing workbook.
- `--parent <selector>`: parent when importing into existing workbook.
- `--if-exists error|skip|merge|replace|rename`: duplicate handling.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`, `--validate-after`, `--json`.

## Output

```json
{
  "ok": true,
  "command": "import",
  "result": {
    "output": "roadmap.xmind",
    "summary": {
      "added": 12
    }
  }
}
```

## Errors

- `invalid_tree_input`
- `not_found`
- `ambiguous_selector`
- `write_failed`

## Notes for Agents

`import --into` overlaps with `add-tree`; use `import` when the source format is the main concern or when creating a new workbook.
