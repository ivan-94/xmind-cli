# Output Formats

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Output format contracts for human and agent usage
- Last updated: 2026-05-21

## Human Text

Default text output should be concise and scan-friendly.

Example:

```text
Roadmap
  Q2
    Payment
      Checkout
      Refunds
```

## Command JSON Envelope

`--json` output is the stable automation contract for the whole command envelope:

```json
{
  "ok": true,
  "command": "tree",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "sheet": "Roadmap",
    "root": {
      "id": "topic-root",
      "path": "/",
      "title": "Roadmap"
    }
  }
}
```

## Compact JSON

`compact-json` is useful for agent context windows. It is a payload-shaping mode used with `--json`, for example:

```bash
xmind tree roadmap.xmind --json --format compact-json --fields id,path,title --depth 2
```

The command still returns the standard JSON envelope. Only the nested `result` payload is compacted.

## Payload Format

Some commands also accept `--format`, such as `export --format markdown` or `export --format json`. That is the command payload format. It is distinct from `--json`, which controls the command envelope.

## `--json` with Payload Formats

When `--json` and `--format` are both present, `--json` wraps the command result in the standard envelope and `--format` controls the nested payload.

Example:

```bash
xmind export roadmap.xmind --format markdown --json
```

returns:

```json
{
  "ok": true,
  "command": "export",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "format": "markdown",
    "content": "# Roadmap\n\n## Q2\n"
  }
}
```

Without `--json`, `export --format markdown` writes raw Markdown to stdout.

## Markdown

Markdown output is used by `export` and outline workflows:

```md
# Roadmap

## Q2

### Payment
```

## Diff

Diff output is used for dry runs and `diff`:

```diff
 /Q2
+  Payment
+    Checkout
-  Old payment
```
