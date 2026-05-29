# xmind export

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for exporting workbook content
- Last updated: 2026-05-22

## Purpose

Export a workbook to a portable representation.

## Synopsis

```bash
xmind export [options] <workbook.xmind>
```

## Options

- `--output <file>`: write to file.
- `--overwrite`: replace existing output file or asset directory.
- Global output and sheet options are documented in `../global-options.md`.

## Output

If `--output` is omitted, export writes to stdout.

If `--output` exists, export fails with `write_failed` unless `--overwrite` is present.

With `--json`, exported content is wrapped in the standard envelope. For text payloads, `result.content` contains the payload string. For `--output`, `result.output` contains the written path and `content` is omitted.

Markdown export renders the selected sheet as heading outline. Topic hyperlinks are preserved as Markdown links with angle-bracket destinations:

```md
# Roadmap

## Q2

### [Payment](<https://example.com/payments>)
```

Export remains scoped to one selected sheet. Use `--sheet`, `--sheet-id`, or `--sheet-index` to choose a non-default sheet in a multi-sheet workbook.

## Errors

- `not_found`
- `ambiguous_selector`
- `unsupported_asset_type`
- `write_failed`

## Notes for Agents

Use Markdown export for human review and JSON export for later automated patch generation. Use `--format assets --output <dir>` to export workbook image resources. Whole-workbook Markdown export is deferred to a future `--all-sheets` option.
