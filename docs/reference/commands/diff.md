# xmind diff

## Source Manifest

- Conversation: XMind CLI product design discussion
- GitHub issue #20: Finalize diff command contract and implementation
- GitHub issue #23: final documentation synchronization
- Scope: Command reference for the current single-workbook diff surface
- Last updated: 2026-05-23

## Purpose

Show structural differences for the workbook in the currently implemented diff surface.
The current public input mode is a single workbook path; it validates and loads the
workbook, applies any global sheet selector, and reports the structural changes
visible to this surface. It does not compare two workbooks.

## Synopsis

```bash
xmind diff [options] <workbook.xmind>
```

## Options

- Global output and sheet options are documented in `../global-options.md`.

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
      "added": 0,
      "updated": 0,
      "deleted": 0,
      "moved": 0
    },
    "changes": []
  }
}
```

Human output is concise:

```text
roadmap.xmind: no changes
```

## Errors

- `file_not_found`
- `parse_failed`
- `invalid_usage`
- `sheet_not_found`
- `ambiguous_sheet`

## Notes for Agents

Use `patch --dry-run --json` when reviewing generated patch files before applying them.
Workbook-vs-workbook or planned-operation review requires a future compare-specific
input mode; `diff` itself intentionally remains the single-workbook surface
documented above.
