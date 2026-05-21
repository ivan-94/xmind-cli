# Patch Schema

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Documentation schema for patch files
- Last updated: 2026-05-21

## Top-Level Shape

```yaml
ops:
  - op: string
```

## Common Operation Fields

| Field | Type | Notes |
| --- | --- | --- |
| `op` | string | Operation name |
| `node` | selector | Topic target |
| `parent` | selector | Parent target |
| `to` | selector | Destination target |
| `position` | string | Child order |
| `tree` | TopicTree | Tree input |
| `if_exists` | string | `error`, `skip`, `merge`, `replace`, `rename` |
| `match_by` | string | `id`, `path`, `title`, `title_path` |
| `operation_index` | number | Returned in errors, not supplied by users |

## Single Topic Operations

```yaml
ops:
  - op: add
    parent: path:/Roadmap/Q2
    title: Payment

  - op: set
    node: path:/Roadmap/Q2/Payment
    note: Payment scope

  - op: delete
    node: path:/Roadmap/Q2/Old payment
```

## Tree Operations

```yaml
ops:
  - op: add_tree
    parent: path:/Roadmap/Q2
    tree:
      title: Payment
      children:
        - title: Checkout
```

## Assertions

```yaml
ops:
  - op: assert_exists
    node: path:/Roadmap/Q2

  - op: assert_not_exists
    node: path:/Roadmap/Q2/Deprecated
```

## Validation

A patch is invalid if operations are missing required fields, selectors are malformed, operation names are unknown, or tree inputs fail validation.

Per-operation semantics are documented in `reference/patch-operations.md`.
