# Exit Codes

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Process exit code policy
- Last updated: 2026-05-21

## Codes

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | Generic failure |
| 2 | Invalid command usage or invalid arguments |
| 3 | File not found or unreadable |
| 4 | Workbook parse failure |
| 5 | Selector not found |
| 6 | Ambiguous selector |
| 7 | Input validation failure |
| 8 | Patch conflict |
| 9 | Workbook validation failure |
| 10 | Write failure |
| 11 | Unsupported feature or asset type |

## JSON Errors

When `--json` is used, nonzero exits write a structured error object to stdout. Human diagnostics may go to stderr. The error schema is documented in `schemas/error.schema.md`.

## Agent Notes

Agents should branch on the JSON `error.code` first and the process exit code second. Exit codes are coarse; error codes carry the actionable details.
