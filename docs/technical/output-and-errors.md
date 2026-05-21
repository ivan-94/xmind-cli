# Output and Errors

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: JSON envelope, human rendering, diagnostics, and exit behavior
- Last updated: 2026-05-21

## JSON Envelope

All `--json` output should serialize one envelope shape:

```rust
pub struct CommandEnvelope<T> {
    pub ok: bool,
    pub command: String,
    pub workbook: Option<PathBuf>,
    pub dry_run: bool,
    pub applied: bool,
    pub result: Option<T>,
    pub error: Option<CliErrorBody>,
    pub warnings: Vec<CliWarning>,
}
```

Read commands use:

```text
dry_run: false
applied: false
```

Dry-run write commands use:

```text
dry_run: true
applied: false
```

Applied write commands use:

```text
dry_run: false
applied: true
```

## Error Body

```rust
pub struct CliErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    pub suggested_fix: String,
    pub selector: Option<String>,
    pub candidates: Vec<CandidateDto>,
    pub operation_index: Option<usize>,
    pub operation: Option<String>,
    pub field_path: Option<String>,
    pub path: Option<String>,
    pub exit_code: i32,
    pub details: serde_json::Value,
}
```

Error code should be an enum with `serde(rename_all = "snake_case")`.

## Human Output

Human output is rendered from the same DTOs used by JSON output. Do not create separate command execution paths for human output.

Human output goals:

- short by default,
- deterministic,
- no color when `--no-color`,
- no progress noise when `--quiet`,
- useful diffs for dry runs.

## `--json` and `--format`

`--json` controls envelope rendering.

`--format` controls payload rendering:

- `tree --json --format compact-json` returns an envelope with compact `result`.
- `export --format markdown --json` returns markdown in `result.content`.
- `export --format markdown` without `--json` writes raw markdown.

## Structured Warnings

Warnings should not be plain strings. They should include stable codes:

```rust
pub struct CliWarning {
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}
```

Examples:

- unsupported preserved feature,
- unknown marker preserved,
- asset not garbage-collected after reference removal.

## Exit Codes

Map typed errors to documented exit codes at the binary boundary. JSON output should still be emitted before exiting nonzero.

## Snapshot Testing

Every command's JSON envelope should have snapshot tests for:

- success,
- dry-run,
- applied write,
- ambiguous selector,
- invalid patch operation,
- validation failure.

