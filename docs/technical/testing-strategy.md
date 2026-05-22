# Testing Strategy

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Unit, integration, snapshot, property, and fixture testing strategy
- Last updated: 2026-05-22

## Test Pyramid

```text
property tests
  selector/path escaping
  query parser
  patch invariants

unit tests
  domain model
  selector resolution
  patch operation semantics
  diff generation

fixture integration tests
  XMind package read/write
  assets
  preservation
  validation

CLI snapshot tests
  JSON envelopes
  human output
  error output
```

## Unit Tests

Core unit tests should cover:

- `TopicPath` parse/render round trips,
- selector parsing,
- query parser precedence,
- position parsing,
- root operation rejection,
- create-missing-path defaults,
- delete modes,
- copy id regeneration,
- merge strategies,
- error code mapping.

## Integration Tests

Integration tests should use committed fixtures:

```text
tests/fixtures/xmind/minimal.xmind
tests/fixtures/xmind/multiple-sheets.xmind
tests/fixtures/xmind/metadata.xmind
tests/fixtures/xmind/topic-image.xmind
```

Each fixture should have a short README describing the intended structure. Avoid large real-world files in the default test suite.

## Snapshot Tests

Use `insta` for JSON and human output snapshots.

Snapshot policy:

- include stable ids in fixtures,
- redact timestamps and temp paths,
- sort object keys where possible,
- avoid snapshots of binary package bytes.

## Property Tests

Use `proptest` for:

- path segment escaping and unescaping,
- selector parse/render round trips,
- query AST parse/render round trips,
- patch dry-run not mutating original model,
- copy producing no duplicate ids when `preserve_ids` is false.

## Golden Write Tests

For write behavior:

1. Copy fixture to temp directory.
2. Run CLI command.
3. Re-open output workbook.
4. Assert domain model changes.
5. Assert unknown fields/assets are preserved.
6. Assert validation passes.

Do not compare zip files byte-for-byte because entry order and timestamps may vary.

## Failure Tests

Every documented error code should have at least one CLI-level test that asserts:

- exit code,
- JSON `error.code`,
- `retryable`,
- `suggested_fix`,
- relevant context fields.

## Non-Goals for Default Tests

Default tests should not require:

- network access,
- installed XMind app,
- user-specific files,
- external cloud storage,
- wall-clock timing assertions.
