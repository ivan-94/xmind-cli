# Command Runtime

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Command execution lifecycle and runtime policies
- Last updated: 2026-05-21

## Runtime Contract

Every command should follow one of these flows:

```text
read command:
  parse args -> build request -> execute app service -> render envelope

write command:
  parse args -> require dry-run/apply -> build request -> plan change -> dry-run or apply -> render envelope

export command:
  parse args -> build export payload -> stdout or output path -> render payload or envelope
```

## CLI Parsing

Use `clap` required argument groups for write commands:

```rust
#[group(required = true, multiple = false)]
struct ApplyMode {
    dry_run: bool,
    apply: bool,
}
```

Invalid combinations should fail before loading the workbook and should render `invalid_usage`.

## Command Request Objects

Each subcommand should convert into a request struct:

```rust
pub struct AddTreeRequest {
    pub workbook: PathBuf,
    pub sheet: SheetSelector,
    pub parent: Selector,
    pub input: TreeInputSource,
    pub position: Position,
    pub mode: MutationMode,
    pub backup: BackupPolicy,
    pub validate_after: bool,
    pub output: OutputPolicy,
}
```

Application services accept request structs, not raw clap matches.

## Position Parsing

`Position` should be typed:

```rust
pub enum Position {
    First,
    Last,
    Index(usize),
    Before(Selector),
    After(Selector),
}
```

The parser accepts one shell argument:

```bash
--position 'before:path:/Q2/Old payment'
--position 'after:title:"Payment"'
```

## Quiet and Color

`--quiet` suppresses human progress messages only. It must not suppress:

- JSON stdout,
- structured errors,
- requested export payloads,
- nonzero exit status.

`--no-color` affects only human-readable output.

## Exit Handling

The binary boundary should be the only layer that calls `std::process::exit`.

Application services return:

```rust
Result<CommandEnvelope, CliDiagnostic>
```

The binary renders the envelope or error envelope, then exits with the mapped code.

## No Hidden Writes

All write services receive `MutationMode`:

```rust
pub enum MutationMode {
    DryRun,
    Apply,
}
```

No lower layer should infer write behavior from missing flags.

