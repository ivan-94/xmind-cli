# Global Options

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Options shared across commands
- Last updated: 2026-05-21

## Output Options

```text
--json
--format text|compact-json|markdown|outline
--fields <comma-separated-fields>
--quiet
--no-color
```

`--json` is the primary automation contract and means the whole command result uses the standard JSON envelope. `--format` controls command-specific payload formats, not the command envelope. `--quiet` suppresses nonessential human output.

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
--backup-dir <dir>
--validate-after
--preserve-unknown
--in-place
--output <file>
```

Every workbook-mutating command requires exactly one of `--dry-run` or `--apply`; omitting both is invalid. The full contract is documented in `mutation-semantics.md`.

## Batch Options

```text
--if-exists error|skip|merge|replace|rename
--match-by id|path|title|title_path
--create-missing-path
--position first|last|index:N|before:<selector>|after:<selector>
```

## Limits

```text
--depth <n>
--limit <n>
--offset <n>
```

Useful for large maps and context-limited agent runs.
