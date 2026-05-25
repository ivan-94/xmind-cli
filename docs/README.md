# XMind CLI Documentation

## Purpose

This documentation describes the intended external shape of `xmind`, a CLI for reading, querying, editing, validating, and exporting XMind files in a way that is native to agents and still comfortable for humans.

The CLI treats an XMind file as a structured workbook, not as a zip archive or storage format. Users and agents operate on sheets, topic trees, topics, metadata, relationships, and batch patches through stable commands and machine-readable outputs.

## Documentation Map

- [product/](product/): product vision, principles, scope, and agent-first design goals.
- [concepts/](concepts/): domain concepts shared by all commands, including selectors, query grammar, paths, tree input, Markdown outline mapping, asset handling, compatibility, patch semantics, and safety rules.
- [reference/](reference/): command reference, legal fields, mutation semantics, patch operations, global options, output contracts, errors, and exit codes.
- [schemas/](schemas/): documented schemas for tree input, patch operations, selectors, command output, and errors.
- [guides/](guides/): workflows for agents and humans.
- [examples/](examples/): copyable inputs and representative outputs.
- [design/](design/): command taxonomy, naming rules, compatibility policy, and future ideas.
- [technical/](technical/): Rust implementation architecture, stack choices, quality gates, E2E strategy, testing strategy, and roadmap.
- [technical/release-policy.md](technical/release-policy.md): versioning, changelog, GitHub Release notes, and checksum rules.
- [installation.md](installation.md): Cargo source install, GitHub Release binary install, install script, release build, release checksum verification, and shell completion setup.

## Reading Order

For product design review:

1. [product/vision.md](product/vision.md)
2. [product/principles.md](product/principles.md)
3. [product/agent-friendly-cli.md](product/agent-friendly-cli.md)
4. [concepts/domain-model.md](concepts/domain-model.md)
5. [design/command-taxonomy.md](design/command-taxonomy.md)

For implementing or using commands:

1. [reference/cli-overview.md](reference/cli-overview.md)
2. [reference/global-options.md](reference/global-options.md)
3. [concepts/selectors.md](concepts/selectors.md)
4. [concepts/query-selectors.md](concepts/query-selectors.md)
5. [reference/fields.md](reference/fields.md)
6. [concepts/tree-input.md](concepts/tree-input.md)
7. [concepts/markdown-outline.md](concepts/markdown-outline.md)
8. [concepts/compatibility-matrix.md](concepts/compatibility-matrix.md)
9. [reference/mutation-semantics.md](reference/mutation-semantics.md)
10. [reference/patch-operations.md](reference/patch-operations.md)
11. [reference/commands/](reference/commands/)

For agent automation:

1. [guides/agent-recipes.md](guides/agent-recipes.md)
2. [guides/safe-editing-workflow.md](guides/safe-editing-workflow.md)
3. [guides/idempotent-workflow.md](guides/idempotent-workflow.md)
4. [schemas/](schemas/)
5. [examples/](examples/)

For Rust implementation:

1. [technical/README.md](technical/README.md)
2. [technical/architecture.md](technical/architecture.md)
3. [technical/tech-stack.md](technical/tech-stack.md)
4. [technical/crate-layout.md](technical/crate-layout.md)
5. [technical/data-model.md](technical/data-model.md)
6. [technical/command-runtime.md](technical/command-runtime.md)
7. [technical/xmind-storage.md](technical/xmind-storage.md)
8. [technical/patch-engine.md](technical/patch-engine.md)
9. [technical/output-and-errors.md](technical/output-and-errors.md)
10. [technical/quality-gates.md](technical/quality-gates.md)
11. [technical/testing-strategy.md](technical/testing-strategy.md)
12. [technical/e2e-test-plan.md](technical/e2e-test-plan.md)
13. [technical/implementation-roadmap.md](technical/implementation-roadmap.md)

For local installation:

1. [installation.md](installation.md)
2. [reference/commands/completion.md](reference/commands/completion.md)

For release policy:

1. [technical/release-policy.md](technical/release-policy.md)
2. [installation.md](installation.md)

## Product Posture

`xmind` should behave like a small domain-specific database editor for mind maps:

- Query before editing.
- Never guess when a selector is ambiguous.
- Preview writes before applying them.
- Preserve unknown XMind data by default.
- Emit stable JSON for agents.
- Keep human-readable output concise and diff-oriented.
