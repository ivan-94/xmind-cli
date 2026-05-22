# Crate Layout

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Proposed Rust source tree and module boundaries
- Last updated: 2026-05-22

## Initial Layout

```text
Cargo.toml
rust-toolchain.toml
rustfmt.toml
clippy.toml
deny.toml

src/
  main.rs
  cli/
    mod.rs
    args.rs
    commands.rs
    output.rs
  app/
    mod.rs
    patch.rs
    set_image.rs
    tree_input.rs
  domain/
    mod.rs
    workbook.rs
    sheet.rs
    topic.rs
    path.rs
    selector.rs
    query.rs
    patch.rs
    diff.rs
    errors.rs
  infra/
    mod.rs
    xmind/
      mod.rs
      package.rs
      decode.rs
      encode.rs
      preserve.rs
      assets.rs
    markdown/
      mod.rs
      outline.rs
      frontmatter.rs
    fs/
      mod.rs
      transaction.rs
      backup.rs
  render/
    mod.rs
    json.rs
    text.rs
    markdown.rs
    diff.rs

tests/
  cli/
  fixtures/
    xmind/
    patch/
  cli/snapshots/
```

## Module Responsibilities

### `cli`

Owns command-line parsing and conversion to application requests.

It must not:

- decode XMind packages,
- mutate workbooks,
- implement selector logic.

### `app`

Owns use-case orchestration. This is where commands become workflows.

Examples:

- load workbook,
- call selector resolver,
- run mutation plan,
- call transaction writer,
- return output DTO.

### `domain`

Owns stable product semantics.

Important types:

- `Workbook`
- `Sheet`
- `Topic`
- `TopicId`
- `TopicPath`
- `Selector`
- `QueryExpr`
- `PatchOp`
- `Diff`
- `Diagnostic`

### `infra`

Owns interaction with external formats and the filesystem.

The XMind reader/writer should preserve unknown package entries and unknown JSON fields through explicit preservation structures.

### `render`

Owns output formatting. JSON rendering should serialize typed DTOs; human output should be derived from the same DTOs, not from separate command logic.

## Test Layout

```text
tests/fixtures/
  xmind/
    minimal.xmind
    multiple-sheets.xmind
    metadata.xmind
    topic-image.xmind
  markdown/
    heading-outline.md
    list-outline.md
  patch/
    add-tree.yaml
    merge-tree.yaml

tests/cli/
  tree_test.rs
  patch_test.rs
  runtime_errors_test.rs
  doc_examples_test.rs
```

Fixtures should be small, committed, and deterministic.
