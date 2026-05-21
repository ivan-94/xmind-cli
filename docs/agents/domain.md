# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Layout

This is a single-context repo.

Primary domain documentation, when present:

- `CONTEXT.md` at the repo root
- `docs/adr/` for architectural decisions

This repo also has existing documentation that should be used as task-relevant context:

- `docs/README.md`
- `docs/product/`
- `docs/concepts/`
- `docs/design/`
- `docs/guides/`
- `docs/reference/`
- `docs/schemas/`
- `docs/examples/`

## Before exploring, read these

- Read `CONTEXT.md` first if it exists.
- Read relevant ADRs under `docs/adr/` if they exist.
- If `CONTEXT.md` or `docs/adr/` do not exist, proceed silently and use the existing `docs/` material that matches the task.
- Do not suggest creating missing domain docs upfront. The producer skill (`/grill-with-docs`) creates them lazily when terms or decisions actually get resolved.

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md` when available. Do not drift to synonyms the glossary explicitly avoids.

If the concept you need is not documented yet, either reconsider whether the project already uses different language or note it as a gap for `/grill-with-docs`.

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding.

## Source Manifest

### Sources

- User confirmation in this setup session: use single-context layout.
- Skill template: `/Users/ivan/.agents/skills/setup-matt-pocock-skills/domain.md`.
- Repository exploration on 2026-05-21: no root `CONTEXT.md`, no `CONTEXT-MAP.md`, no `docs/adr/`, and existing documentation under `docs/`.

### Produced artifacts

- `docs/agents/domain.md`
- `AGENTS.md`

### Key decisions

- Treat this repository as single-context.
- Use existing `docs/` product, concept, design, guide, reference, schema, and example material as task-relevant context until a root `CONTEXT.md` or ADRs exist.

### Verification evidence

- `find . -maxdepth 3 \( -name AGENTS.md -o -name CLAUDE.md -o -name CONTEXT.md -o -name CONTEXT-MAP.md \) -print` found no matching files before setup.
- `find docs -maxdepth 4 -type d \( -path '*/adr' -o -path '*/agents' \) -print` found no existing `docs/adr/` or `docs/agents/` before setup.
- `find docs -maxdepth 3 -type f -print` showed existing product, concept, design, guide, reference, schema, and example docs.

### Open questions / risks

- If this repo becomes a monorepo or gains separate sub-domain contexts, switch to a `CONTEXT-MAP.md` driven multi-context layout.
