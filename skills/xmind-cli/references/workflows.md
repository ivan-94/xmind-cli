# xmind-cli Workflows

Use this file for concrete task execution. Keep commands exact, preserve the
dry-run/apply split, and replace placeholders with task-specific paths,
selectors, sheet names, and input files.

## Table of contents

- Environment setup
- Workbook reconnaissance
- Target resolution
- Safe single edits
- Subtree generation
- Batch patching
- Import and export
- Recovery
- Acceptance checklist

## Environment setup

Choose the command prefix before starting:

```bash
xmind --version
```

If `xmind` is not installed but you are in the repository checkout:

```bash
cargo run -- --version
```

For repeated local development commands, build once:

```bash
cargo build --workspace
target/debug/xmind --version
```

Use the same command prefix consistently in examples below.

## Workbook reconnaissance

Start read-only. This establishes format support, sheets, topic shape, and
whether validation already fails before your change.

```bash
xmind inspect plan.xmind --json
xmind sheets plan.xmind --json
xmind validate plan.xmind --json
```

Read a bounded tree:

```bash
xmind tree plan.xmind --sheet "Roadmap" --depth 2 --json
```

For large workbooks, shape the payload:

```bash
xmind tree plan.xmind \
  --sheet "Roadmap" \
  --depth 3 \
  --json \
  --format compact-json \
  --fields id,path,title
```

## Target resolution

Use human terms for discovery:

```bash
xmind find plan.xmind --sheet "Roadmap" --title "Payment" --json
```

Then inspect the exact topic:

```bash
xmind get plan.xmind --node "id:topic-123" --json
```

Rules:

- Prefer `id:` selectors for writes.
- Use `path:` only when the path is known to be unique.
- Use `--sheet`, `--sheet-id`, or `--sheet-index` for multi-sheet workbooks.
- If discovery finds multiple matches, ask the user or use surrounding tree
  context to choose the right `id:`.

## Safe single edits

All write-capable commands must be previewed before apply.

Add one topic:

```bash
xmind add plan.xmind \
  --parent "id:topic-123" \
  --title "Refunds" \
  --dry-run \
  --json
```

Set explicit fields:

```bash
xmind set plan.xmind \
  --node "id:topic-456" \
  --title "Payments" \
  --note "Q2 payment scope" \
  --dry-run \
  --json
```

Delete a topic:

```bash
xmind delete plan.xmind \
  --node "id:topic-789" \
  --dry-run \
  --json
```

Move a subtree:

```bash
xmind move plan.xmind \
  --node "id:topic-789" \
  --to "id:topic-456" \
  --position last \
  --dry-run \
  --json
```

When the dry-run diff is correct, repeat the same command with
`--apply --backup --json`, then validate:

```bash
xmind validate plan.xmind --json
```

## Subtree generation

Use `add-tree` when generated content has hierarchy:

```bash
xmind add-tree plan.xmind \
  --parent "id:topic-123" \
  --input generated-tree.yaml \
  --dry-run \
  --json
```

Apply after review:

```bash
xmind add-tree plan.xmind \
  --parent "id:topic-123" \
  --input generated-tree.yaml \
  --apply \
  --backup \
  --json
xmind validate plan.xmind --json
```

Read `references/examples.md` before writing the tree payload.

## Batch patching

Use `patch` for multi-operation edits that should succeed or fail as one plan.

```bash
xmind patch plan.xmind --ops ops.yaml --dry-run --json
xmind patch plan.xmind --ops ops.yaml --apply --backup --json
xmind validate plan.xmind --json
```

Good patch candidates:

- assert a topic exists before editing it,
- add a subtree and then sort children,
- rename several topics,
- merge generated content into an existing branch,
- delete deprecated descendants only after assertions pass.

Avoid patch for a single obvious edit unless the user explicitly wants a
reviewable operation file.

## Import and export

Export for human review:

```bash
xmind export plan.xmind --format markdown --json
```

Export for structured processing:

```bash
xmind export plan.xmind --format json --json
```

Import creates or updates workbook content, so use the mutation loop:

```bash
xmind import plan.xmind \
  --input outline.md \
  --dry-run \
  --json
```

Then apply with backup and validate.

## Recovery

Create a backup before risky external operations:

```bash
xmind backup plan.xmind --json
```

Restore through the CLI rather than copying files manually:

```bash
xmind restore plan.xmind \
  --backup ".xmind-backups/plan-20260528-120000.xmind" \
  --dry-run \
  --json
```

Then apply and validate when the restore target is correct.

## Acceptance checklist

Before saying the task is done:

- read commands used `--json` when their output drove decisions,
- mutation target was resolved to a stable selector,
- dry-run output was checked before apply,
- applied mutation used `--backup`,
- final JSON showed `applied: true` for writes,
- `xmind validate <file> --json` passed after writes,
- any ambiguity or unsupported format was reported with the CLI error code.
