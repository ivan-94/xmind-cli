# Path Addressing

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Path syntax and behavior for topic addresses
- Last updated: 2026-05-21

## Path Syntax

Topic paths are absolute and slash-delimited:

```text
path:/Root/Feature/Auth
```

`path:/` means the selected sheet root.

## Human Readability

Paths are designed to be copied from `xmind tree --json` or human-readable tree output. They should remain stable unless titles or hierarchy change.

## Duplicate Titles

Sibling topics may share the same title in XMind. If a path segment is duplicated at the same level, the selector is ambiguous.

The CLI may expose disambiguated path hints:

```text
path:/Root/Feature/Auth[2]
```

However, generated scripts should prefer ids when duplicates exist.

## Creating Missing Paths

Write commands that create topics may support:

```bash
--create-missing-path
```

Example:

```bash
xmind add plan.xmind --parent "path:/Root/Q2/Payment" --title "Refunds" --create-missing-path
```

If `Q2` or `Payment` is missing, the CLI creates the missing path segments before adding `Refunds`.

## Canonical Paths

JSON output should include canonical paths after the command resolves or mutates a topic:

```json
{
  "id": "topic-123",
  "path": "/Root/Q2/Payment"
}
```

