# Path Addressing

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Path syntax and behavior for topic addresses
- Last updated: 2026-05-21

## Path Syntax

Topic paths are absolute and slash-delimited:

```text
path:/Feature/Auth
```

`path:/` means the selected sheet root.

Canonical paths are relative to the selected sheet root and never include the sheet title or root topic title. A root topic titled `Roadmap` still has canonical path `/`; its child `Q2` has canonical path `/Q2`.

## Human Readability

Paths are designed to be copied from `xmind tree --json` or human-readable tree output. They should remain stable unless titles or hierarchy change.

## Duplicate Titles

Sibling topics may share the same title in XMind. If a path segment is duplicated at the same level, the selector is ambiguous.

The CLI may expose disambiguated path hints:

```text
path:/Feature/Auth[2]
```

However, generated scripts should prefer ids when duplicates exist.

## Escaping

Paths use `/` as a delimiter. Literal slashes in titles are escaped as `\/`.

Examples:

```text
Title: API/SDK
Path:  path:/API\/SDK
```

If a topic title is exactly `/`, its path segment is `\/`:

```text
path:/\/
```

This is distinct from the root selector:

```text
path:/
```

Backslashes and double quotes inside path selectors should be shell-quoted by quoting the whole selector argument.

## Creating Missing Paths

Write commands that create topics may support:

```bash
--create-missing-path
```

Example:

```bash
xmind add plan.xmind --parent "path:/Q2/Payment" --title "Refunds" --create-missing-path
```

If `Q2` or `Payment` is missing, the CLI creates the missing path segments before adding `Refunds`.

Intermediate topic defaults are defined in `../reference/mutation-semantics.md`.

## Canonical Paths

JSON output should include canonical paths after the command resolves or mutates a topic:

```json
{
  "id": "topic-123",
  "path": "/Q2/Payment"
}
```

## Sheet Scope

Paths are resolved inside a sheet scope. Use `--sheet`, `--sheet-id`, or `--sheet-index` when a workbook has multiple sheets.
