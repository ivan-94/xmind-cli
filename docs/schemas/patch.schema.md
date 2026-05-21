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
| `target` | selector | Merge target selector |
| `parent` | selector | Parent target |
| `to` | selector | Destination target |
| `path` | canonical path | Path value without selector prefix, used by `ensure_path` |
| `position` | string | Child order |
| `fields` | object | Explicit field updates for `set`; `null` clears a nullable field |
| `tree` | TopicTree | Tree input |
| `if_exists` | string | `error`, `skip`, `merge`, `replace`, `rename` |
| `match_by` | string | `id`, `path`, `title`, `title_path` |
| `children_only` | boolean | `delete` only |
| `promote_children` | boolean | `delete` only |
| `preserve_ids` | boolean | `copy` only; defaults to false |
| `prune` | boolean | `merge_tree` only; defaults to false |
| `operation_index` | number | Returned in errors, not supplied by users |

## Operation Names

Agents should generate canonical operation names:

```text
add
add_tree
set
delete
move
copy
replace_tree
merge_tree
ensure_path
sort_children
set_tree_metadata
assert_exists
assert_not_exists
```

Accepted aliases:

```text
delete_tree -> delete
move_tree -> move
clone_tree -> copy
```

## Single Topic Operations

```yaml
ops:
  - op: add
    parent: path:/Q2
    title: Payment

  - op: set
    node: path:/Q2/Payment
    fields:
      note: Payment scope

  - op: delete
    node: path:/Q2/Old payment
    children_only: false
```

## Tree Operations

```yaml
ops:
  - op: add_tree
    parent: path:/Q2
    tree:
      title: Payment
      children:
        - title: Checkout
```

```yaml
ops:
  - op: merge_tree
    target: path:/Q2/Payment
    match_by: title_path
    prune: false
    tree:
      title: Payment
```

## Ensure Path

`ensure_path` uses a canonical path value, not a selector string:

```yaml
ops:
  - op: ensure_path
    path: /Q2/Payment
```

The value intentionally omits the `path:` selector prefix.

## Assertions

```yaml
ops:
  - op: assert_exists
    node: path:/Q2

  - op: assert_not_exists
    node: path:/Q2/Deprecated
```

## Validation

A patch is invalid if operations are missing required fields, selectors are malformed, operation names are unknown, or tree inputs fail validation.

Per-operation semantics are documented in `reference/patch-operations.md`.
