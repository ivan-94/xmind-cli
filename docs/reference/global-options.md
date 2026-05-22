# Global Options

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Options shared across commands
- Last updated: 2026-05-22

## Output Options

```text
--json
--format <format>
--fields <comma-separated-fields>
--quiet
--no-color
```

`--json` is the primary automation contract and means the whole command result uses the standard JSON envelope. `--format` controls command-specific payload formats, not the command envelope. Current help exposes `text`, `json`, `compact-json`, `markdown`, `outline`, and `assets`.

`--quiet` suppresses nonessential human progress text. It must not suppress JSON stdout, structured errors, or requested export payloads. `--no-color` disables ANSI color in human-readable output and has no effect on JSON output.

## Sheet Options

```text
--sheet <title>
--sheet-id <id>
--sheet-index <index>
```

If the workbook has multiple sheets, topic commands should require unambiguous sheet scope.

## Selector Options

```text
--node <selector>
--parent <selector>
--to <selector>
```

Selector syntax is documented in `concepts/selectors.md`.

## Write Options

```text
--dry-run
--apply
--backup
--output <file>
--overwrite
```

Every workbook-mutating command requires exactly one of `--dry-run` or `--apply`; omitting both is invalid. `backup` has command-specific `--backup-dir`; `export` and `import --output` expose command-specific output flags. The full mutation contract is documented in `mutation-semantics.md`.

## Batch Options

```text
--create-missing-path
--position first|last|index:N|before:<selector>|after:<selector>
```

`before:<selector>` and `after:<selector>` are passed as one shell argument. Quote the whole value when it contains shell-sensitive characters:

```bash
--position 'before:path:/Q2/Old payment'
--position 'after:title:"Payment"'
```

Agents should prefer `id:` or `path:` selectors in position values.

## Limits

```text
--depth <n>
--limit <n>
--offset <n>
```

Useful for large maps and context-limited agent runs.
