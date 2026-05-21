# Product Principles

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Top-level principles for all CLI commands and documentation
- Last updated: 2026-05-21

## Principles

### 1. Domain Model First

Users operate on workbooks, sheets, topics, paths, metadata, and patches. They do not operate on zip entries, internal JSON filenames, package manifests, or storage-specific ids unless explicitly requested for diagnostics.

### 2. Query Before Mutation

Every mutation target must be addressable through the same selector model used by read commands. If the target cannot be uniquely resolved, the command fails and returns candidates.

### 3. Dry Run Is a First-Class Mode

All write commands support `--dry-run`. Dry runs must resolve selectors, validate input, compute changes, and return a diff without modifying the file.

### 4. Ambiguity Is an Error

The CLI never silently picks the first matching node. Multiple matches return an `ambiguous_selector` error with enough candidate data for the caller to retry safely.

### 5. Preserve Unknown Data

XMind may store fields that the CLI does not understand yet. By default, edits preserve unknown fields and only modify the minimum necessary subtree.

### 6. Stable JSON for Agents

Every command supports `--json`. JSON keys are stable, explicit, and documented. Agent-facing output must avoid decorative text and unstable formatting.

### 7. Human Output Is a Preview, Not a Protocol

Default output should be readable and compact, but scripts and agents should use `--json`. Human output can evolve more freely than JSON contracts.

### 8. Batch Is Not Just Loops

The CLI should support tree-level and patch-level operations directly. Agents should not need to shell-loop over dozens of single-node commands to express a structured change.

### 9. Idempotence Is a Product Feature

Operations should support `if_exists`, `match_by`, `merge`, and `ensure_path` semantics so agents can safely retry after interruptions.

### 10. Validation Closes the Loop

Write commands should optionally validate after applying changes. `xmind validate` must be a standalone command and a reusable safety step after batch edits.

