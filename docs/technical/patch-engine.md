# Patch Engine

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Patch parsing, normalization, planning, execution, and diagnostics
- Last updated: 2026-05-21

## Responsibilities

The patch engine owns:

- parsing YAML/JSON patch files,
- schema validation,
- operation alias normalization,
- operation ordering,
- selector resolution,
- conflict detection,
- operation execution on a working copy,
- diff generation,
- operation-indexed diagnostics.

## Parse and Normalize

Patch input should parse into DTOs first:

```rust
struct PatchFileDto {
    ops: Vec<PatchOpDto>,
}
```

Then convert into typed domain ops:

```rust
Vec<PatchOp>
```

During conversion:

- `delete_tree` normalizes to `delete`,
- `move_tree` normalizes to `move`,
- `clone_tree` normalizes to `copy`,
- missing booleans default to false,
- `merge_tree.prune` defaults to false,
- `copy.preserve_ids` defaults to false.

Dry-run output should report canonical op names.

## Execution Model

Patch execution uses a working copy:

```text
original workbook
  -> working workbook
  -> op 0
  -> op 1
  -> ...
  -> diff original vs working
```

If any op fails, the original workbook is unchanged and the error includes:

- `operation_index`,
- `operation`,
- `selector` when relevant,
- `field_path` for schema or input errors,
- candidate topics for ambiguity.

## Selector Timing

Selectors are resolved at the time each operation executes against the working copy. This allows later operations to target topics created by earlier operations.

## Conflict Detection

Conflicts include:

- ambiguous selector,
- missing selector,
- root operation not allowed,
- move into self or descendant,
- incompatible duplicate creation,
- `children_only` and `promote_children` both true,
- `match_by: id` with missing input ids,
- `prune: true` deleting topics that another later op targets.

## Idempotent Merge

`merge_tree` should support:

```text
match_by: title_path
match_by: id
match_by: path
match_by: title
```

`title_path` is the recommended agent default. It matches by relative title path from the merge target. It is resilient when ids are not known.

`id` requires ids in tree input and should fail early if any relevant node lacks an id.

## Diff Generation

The patch engine should emit structured diff events while executing operations or by comparing original and working trees after execution. The diff must include enough path/id data for agents to continue from it.

## Atomicity

Patch application is all-or-nothing. The engine never writes files; it returns a planned or applied workbook to the mutation service, which handles transactional write behavior.

