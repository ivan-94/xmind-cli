# xmind validate

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for workbook validation
- Last updated: 2026-05-21

## Purpose

Validate workbook readability and structural integrity.

## Synopsis

```bash
xmind validate <workbook.xmind> [--json]
```

## Options

- `--strict`: treat warnings as failures.
- `--json`: emit structured validation result.

## Checks

- workbook can be opened,
- sheets are readable,
- topic tree has no cycles,
- topic ids are valid where required,
- topic ordering is valid,
- required fields are present,
- unknown fields can be preserved.

## Output

```json
{
  "ok": true,
  "command": "validate",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "valid": true,
    "warnings": [],
    "errors": []
  }
}
```

## Errors

- `file_not_found`
- `parse_failed`
- `validation_failed`

## Notes for Agents

Run this after write operations when the command did not use `--validate-after`.

