# Quality Gates

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Required lint, format, type, test, and security gates
- Last updated: 2026-05-22

## Required Local Gates

The implemented local gate is:

```bash
./scripts/quality-gate.sh
```

It currently runs:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Formatting

Use `rustfmt` with committed `rustfmt.toml`. Do not rely on editor defaults.

Minimum expectation:

```bash
cargo fmt --all
```

## Linting

Use `clippy` with warnings denied in CI:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Recommended code-level settings:

```rust
#![deny(unsafe_code)]
#![warn(missing_debug_implementations)]
```

Do not globally enable `missing_docs` at the beginning; use targeted documentation for public domain types and command contracts first.

## Type Safety

Type safety requirements:

- no raw selector strings after CLI parsing,
- no raw path strings inside domain mutation logic,
- no untyped patch maps after input validation,
- no string error code construction at call sites,
- no direct filesystem writes from domain or patch engine.

## CI Gate

CI should run on pull requests:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Optional release gate:

```bash
cargo test --workspace --all-features --release
```

## Pre-Commit

Use a lightweight pre-commit hook or documented local command. It should run at least:

```bash
cargo fmt --all
cargo test --workspace --lib
```

Full clippy can be slower and belongs in CI and pre-PR checks.

## AI-Native Feedback

Every failed quality gate should be easy for an agent to parse and recover from:

- keep commands deterministic,
- avoid interactive prompts,
- keep fixtures committed,
- avoid network-dependent tests in the default suite,
- separate slow/integration tests behind explicit flags.
