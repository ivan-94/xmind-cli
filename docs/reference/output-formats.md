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
  "result": {
    "sheet": "Roadmap",
    "root": {
      "id": "topic-root",
      "path": "/Roadmap",
      "title": "Roadmap"
    }
  }
}
```

## Compact JSON

`compact-json` is useful for agent context windows. It omits optional fields unless requested by `--fields`.

## Payload Format

Some commands also accept `--format`, such as `export --format markdown` or `export --format json`. That is the command payload format. It is distinct from `--json`, which controls the command envelope.

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
 /Roadmap/Q2
+  Payment
+    Checkout
-  Old payment
```
