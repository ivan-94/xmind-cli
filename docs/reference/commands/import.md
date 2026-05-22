# xmind import

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for importing structured content
- Last updated: 2026-05-22

## Purpose

Create a new workbook or update an existing workbook from Markdown, YAML, or JSON tree input.

## Synopsis

```bash
xmind import [options] --input outline.md --output roadmap.xmind (--dry-run | --apply)
xmind import [options] --input tree.yaml --into roadmap.xmind --parent "path:/Q2" (--dry-run | --apply)
```

## Options

- `--input <file>`: input file.
- `--output <file>`: create output workbook.
- `--into <workbook.xmind>`: import into existing workbook.
- `--parent <selector>`: parent when importing into existing workbook.
- `--overwrite`: replace existing `--output` workbook.
- `--markdown-mode heading|list|hybrid|auto`: Markdown parsing mode.
- `--dry-run` or `--apply`: exactly one is required.
- Global output and sheet options are documented in `../global-options.md`.

## Output

```json
{
  "ok": true,
  "command": "import",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
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

## Output and Overwrite Behavior

`--output` creates a new workbook and fails if the target exists unless `--overwrite` is present. `--into` mutates an existing workbook and follows the normal `--dry-run | --apply` and `--backup` rules.

## Dry Run Behavior

`import --output ... --dry-run` does not create a file. It parses the input, builds the would-be workbook in memory, validates it when requested, and returns a summary plus root tree preview. The diff is reported as a creation diff from an empty workbook.

## Notes for Agents

`import --into` overlaps with `add-tree`; use `import` when the source format is the main concern or when creating a new workbook.
