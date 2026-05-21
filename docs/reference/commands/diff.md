# xmind diff

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for comparing workbooks or planned operations
- Last updated: 2026-05-21

## Purpose

Show structural differences between two workbooks or between a workbook and a patch preview.

## Synopsis

```bash
xmind diff <before.xmind> <after.xmind> [--json]
xmind diff <workbook.xmind> --ops <ops.yaml> [--json]
```

Exactly one diff mode is required:

- two workbook positional arguments, or
- one workbook positional argument plus `--ops`.

`xmind diff <workbook.xmind>` without `--ops` is invalid and fails with `invalid_usage`.

## Options

- `--ops <file>`: show diff that patch would produce.
- `--sheet <title>`: limit diff to one sheet.
- `--format text|markdown`: human payload format.
- `--json`: emit structured diff.

## Output

```json
{
  "ok": true,
  "command": "diff",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "summary": {
      "added": 2,
      "updated": 1,
      "deleted": 0,
      "moved": 1
    },
    "changes": []
  }
}
```

## Errors

- `file_not_found`
- `parse_failed`
- `invalid_patch`
- `invalid_usage`

## Notes for Agents

Use `diff --ops` when reviewing generated patch files before applying them.
