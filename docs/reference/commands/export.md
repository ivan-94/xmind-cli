# xmind export

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

Markdown export renders the selected sheet as a heading outline. Topic notes are written as Markdown body text immediately after their topic heading, before child topic headings. Topic hyperlinks are preserved as Markdown links with angle-bracket destinations:

```md
# Roadmap

## Q2

Q2 delivery scope.

### [Payment](<https://example.com/payments>)

Supports card payments and refund workflows.
```

Export remains scoped to one selected sheet. Use `--sheet`, `--sheet-id`, or `--sheet-index` to choose a non-default sheet in a multi-sheet workbook.

## Errors

- `not_found`
- `ambiguous_selector`
- `unsupported_asset_type`
- `write_failed`

## Notes for Agents

Use Markdown export for human review and JSON export for later automated patch generation. Markdown export includes topic titles, notes, and hyperlinks, but it does not export labels, markers, images, attachments, relationships, summaries, boundaries, or visual style. Use `--format assets --output <dir>` to export workbook image resources. Whole-workbook Markdown export is deferred to a future `--all-sheets` option.
