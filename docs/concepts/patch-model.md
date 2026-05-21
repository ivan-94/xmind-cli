# Patch Model

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Declarative batch operation model for xmind patch
- Last updated: 2026-05-21

## Purpose

Patch files describe a sequence of operations to apply to a workbook. They are the preferred interface for complex agent changes.

## Patch Shape

```yaml
ops:
  - op: add_tree
    parent: path:/Roadmap/Q2
    position: last
    if_exists: merge
    match_by: title_path
    tree:
      title: 支付能力
      children:
        - title: 收银台
        - title: 退款
```

## Operation Classes

- Read-like validation: `assert_exists`, `assert_not_exists`
- Single topic mutation: `add`, `set`, `delete`, `move`, `copy`
- Tree mutation: `add_tree`, `replace_tree`, `merge_tree`, `delete_tree`, `move_tree`, `clone_tree`
- Structure helpers: `ensure_path`, `sort_children`
- Metadata helpers: `set_tree_metadata`

## Ordering

Operations run in file order. Later operations can target topics created by earlier operations.

## Atomicity

The preferred default is atomic patch application:

1. Load workbook.
2. Resolve and validate all operations in a working copy where possible.
3. Compute diff.
4. Apply all operations.
5. Validate output when requested.
6. Write the workbook.

If any operation fails, no partial file write occurs.

## Idempotence

Patch operations may include:

```yaml
if_exists: error | skip | merge | replace | rename
match_by: id | path | title | title_path
```

Agents should use `merge` plus `title_path` when they need retryable tree updates.

## Dry Run

`xmind patch --dry-run` must return the same resolution and diff that `--apply` would use, without writing the file.

