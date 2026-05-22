# Architecture

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: High-level Rust architecture and data flow
- Last updated: 2026-05-22

## Goals

The architecture must satisfy the existing product contracts:

- deterministic command behavior,
- stable JSON envelopes,
- explicit `--dry-run` / `--apply`,
- selector-driven reads and writes,
- declarative patch execution,
- preservation of unknown XMind data,
- atomic writes with validation,
- and complete automated quality gates.

## Layered Architecture

```text
CLI binary
  argument parsing
  command dispatch
  terminal policy

Application services
  inspect service
  query service
  mutation service
  patch service
  import/export service
  validation service

Domain core
  workbook model
  sheet model
  topic tree model
  selectors
  query AST
  patch operations
  diffs
  diagnostics

Infrastructure
  XMind package reader/writer
  JSON/YAML/Markdown codecs
  asset storage
  filesystem transaction writer
```

## Main Data Flow

Read command:

```text
CLI args
  -> command request
  -> load workbook package
  -> decode into domain model plus preservation bag
  -> resolve sheet and selector
  -> project requested fields
  -> render text or JSON envelope
```

Write command:

```text
CLI args
  -> command request
  -> load workbook package
  -> decode into domain model plus preservation bag
  -> resolve all selectors
  -> build change plan
  -> compute diff
  -> dry-run: return diff only
  -> apply: write candidate package to temp path
  -> validate candidate
  -> backup original if requested
  -> atomic replace original
  -> return JSON envelope
```

Patch command:

```text
patch file
  -> parse YAML/JSON
  -> validate schema
  -> normalize aliases to canonical ops
  -> resolve and execute ops against working copy
  -> collect operation-level diagnostics
  -> compute aggregate diff
  -> dry-run or apply through mutation service
```

## Crate Boundary

The implemented project is currently one Cargo package with one binary target:

- `xmind`: binary entrypoint at `src/main.rs`.
- internal modules under `src/cli`, `src/app`, `src/domain`, `src/infra`, and `src/render`.

If the implementation grows, split storage and domain into separate crates later. Do not over-split before the first full command path is proven.

## Dependency Direction

```text
cli -> app -> domain
app -> infra
infra -> domain
domain -> no project-local outer layers
```

The domain layer must not depend on CLI parsing, terminal output, filesystem paths, or concrete XMind package internals.

## AI-Native Feedback Loop

Every command should produce enough machine-readable context for an agent to recover:

- command name,
- workbook path,
- `dry_run`,
- `applied`,
- summary,
- diff,
- resolved selectors,
- operation index for patch failures,
- candidates for ambiguous selectors,
- field paths for schema errors.

Human output is secondary. JSON output is the source of truth for automation.
