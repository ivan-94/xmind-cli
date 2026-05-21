# Tech Stack

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Rust crate choices and rationale
- Last updated: 2026-05-21

## Language

Use stable Rust, edition 2024 if the initial toolchain supports it cleanly; otherwise edition 2021. Prefer boring, well-maintained crates with strong ecosystem adoption.

## CLI

Use `clap` with derive support.

Rationale:

- mature CLI parsing,
- excellent help text,
- enum value parsing,
- conflicts and required groups for `--dry-run` / `--apply`,
- shell completion support later.

## Serialization

Use:

- `serde`
- `serde_json`
- `serde_yaml` or `serde_yml`

All public JSON structs should derive `Serialize`. Input DTOs should derive `Deserialize` and then convert into validated domain types.

## Errors

Use:

- `thiserror` for typed library errors,
- `miette` for rich human diagnostics if useful,
- custom `CliErrorEnvelope` for JSON output.

Avoid `anyhow` in the domain and application layers. `anyhow` may be acceptable only at the binary boundary during early scaffolding, but typed errors should be the implementation target.

## XMind Package IO

Use:

- `zip` for `.xmind` package read/write,
- `serde_json` for XMind JSON content,
- `time` for timestamps,
- `sha2` for asset checksums.

The first implementation should focus on modern XMind JSON package formats. Older package variants can be detected and returned as `unsupported_format` until explicitly supported.

## Markdown

Use:

- `pulldown-cmark` for Markdown event parsing,
- `serde_yaml` for frontmatter parsing.

Do not parse Markdown outlines with ad hoc line splitting except for isolated, tested helpers after the event stream has identified blocks.

## Query Parsing

Use `pest`, `chumsky`, or a small hand-written Pratt parser.

Recommendation: start with `chumsky` or a hand-written parser because the grammar is small and error messages need field/operator context. The parser must produce a typed query AST.

## Diffing

Implement a domain-specific tree diff rather than using a text diff library for the core diff.

Human diff output can be rendered from structured diff events:

```rust
enum DiffEvent {
    Added { path: TopicPath },
    Removed { path: TopicPath },
    Updated { path: TopicPath, fields: Vec<FieldChange> },
    Moved { from: TopicPath, to: TopicPath },
}
```

## Testing

Use:

- built-in `cargo test`,
- `insta` for snapshot tests,
- `assert_cmd` for CLI tests,
- `predicates` for command assertions,
- `tempfile` for filesystem tests,
- `proptest` for selector/path escaping and patch invariants.

## Lint, Format, and Security

Required local gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
cargo audit
cargo deny check
```

`cargo audit` and `cargo deny` may be added after the initial `Cargo.toml` exists, but the project should be designed to include them before first release.

