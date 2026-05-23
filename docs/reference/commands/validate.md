# xmind validate

## Source Manifest

- Conversation: XMind CLI product design discussion
- GitHub issue #21: validate structural diagnostics
- GitHub issue #23: final documentation synchronization
- Scope: Command reference for workbook validation
- Last updated: 2026-05-23

## Purpose

Validate workbook readability and structural integrity.

## Synopsis

```bash
xmind validate [options] <workbook.xmind>
```

## Options

- `--strict`: treat warnings as failures.
- Global output and sheet options are documented in `../global-options.md`.

## Checks

- workbook can be opened,
- sheets are readable,
- `content.json` and supported modern package structure are present,
- required sheet and root-topic fields are present,
- duplicate topic ids are reported with stable structural paths,
- relationship endpoints reference known topics,
- image references point at existing package resources,
- unknown fields can be preserved.

`--strict` turns warnings into `validation_failed`. The current validator does
not attempt full XMind App visual/layout validation or every private relationship
variant.

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

Run this after write operations when you need an explicit validation result.
