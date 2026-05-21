# Query Selectors

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Query selector grammar and agent-safe generation rules
- Last updated: 2026-05-21

## Purpose

`query:` selectors are for searches and audits. They are more expressive than `title:` but must remain small enough for agents to generate reliably.

## Grammar

```text
query:<expr>

expr        := or_expr
or_expr     := and_expr ("or" and_expr)*
and_expr    := not_expr ("and" not_expr)*
not_expr    := "not" not_expr | primary
primary     := comparison | "(" expr ")"
comparison  := field operator value
operator    := "=" | "!=" | ">" | ">=" | "<" | "<=" | "contains" | "starts_with" | "ends_with" | "in" | "exists"
value       := string | number | boolean | "[" value ("," value)* "]"
```

Operator precedence is:

1. parentheses,
2. `not`,
3. `and`,
4. `or`.

## Fields

| Field | Type | Operators |
| --- | --- | --- |
| `id` | string | `=`, `!=`, `in` |
| `title` | string | `=`, `!=`, `contains`, `starts_with`, `ends_with`, `in` |
| `note` | string | `=`, `!=`, `contains`, `exists` |
| `path` | canonical path | `=`, `!=`, `contains`, `starts_with`, `in` |
| `label` | string collection | `=`, `!=`, `contains`, `in`, `exists` |
| `marker` | string collection | `=`, `!=`, `contains`, `in`, `exists` |
| `hyperlink` | string | `=`, `!=`, `contains`, `exists` |
| `image` | object | `exists` |
| `depth` | number | `=`, `!=`, `>`, `>=`, `<`, `<=`, `in` |
| `children_count` | number | `=`, `!=`, `>`, `>=`, `<`, `<=`, `in` |

## Strings

Strings use double quotes. Escape backslash and double quote:

```text
query:title = "He said \"Pay\""
query:path = "/Q2/API\\/SDK"
```

## Examples

```text
query:title contains "Payment" and marker = "priority-1"
query:(label = "MVP" or marker = "priority-1") and depth <= 3
query:image exists
query:not note exists
query:path starts_with "/Q2" and title != "Deprecated"
```

## Agent Rules

Agents should prefer simple `and` expressions and avoid `or` unless necessary. Use `find --query` for discovery; do not use a broad `query:` selector with mutating commands unless the command explicitly supports multi-target behavior.
