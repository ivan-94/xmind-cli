# Issue Tracker: GitHub Issues

GitHub Issues is the formal issue and PRD tracker for this repo.

Canonical repository:

- `git@github.com:ivan-94/xmind-cli.git`
- `https://github.com/ivan-94/xmind-cli`

## Conventions

- Use GitHub Issues for PRDs, implementation slices, bug reports, release tasks, and E2E coverage work.
- Keep `PLAN.md` as the repository-level backlog and source-of-truth checklist for the current implementation program.
- Link issues back to the relevant `PLAN.md` phase or documentation section.
- Use `.scratch/` only for temporary agent workspaces, drafts, generated prompts, or local handoff notes that are not the formal tracker.
- When a task comes from a PRD or plan, preserve the original sources in the issue body under `Source Manifest`.
- Close issues through pull requests where possible, using `Closes #<issue-number>` or `Fixes #<issue-number>`.

## When a skill says "publish to the issue tracker"

Create or update a GitHub Issue in `ivan-94/xmind-cli`.

The issue body should include:

- problem or goal,
- scope,
- non-goals if relevant,
- acceptance criteria,
- suggested implementation notes,
- test or E2E expectations,
- `Source Manifest`.

## When a skill says "fetch the relevant ticket"

Read the referenced GitHub Issue or PR first. If the user gives only an issue number, resolve it in `ivan-94/xmind-cli`.

## Source Manifest

### Sources

- User alignment on 2026-05-23: the canonical remote is `git@github.com:ivan-94/xmind-cli.git`, the repository already exists, it is public, and the formal tracker should move from local markdown to GitHub Issues.
- Previous local markdown convention in this file.
- `PLAN.md`: current backlog and future GitHub infrastructure slices.
- `~/.agents/docs/agents/handoff-policy.md`: Source Manifest requirements for durable handoff artifacts.

### Produced artifacts

- `docs/agents/issue-tracker.md`
- `PLAN.md`
- `docs/technical/e2e-test-plan.md`
- `implementation-notes.html`

### Key decisions

- GitHub Issues replaces `.scratch/<feature-slug>/` as the formal tracker.
- `.scratch/` remains acceptable for temporary local drafts and agent working files.
- Issues should preserve Source Manifest data so downstream agents can reread original sources.

### Verification evidence

- `git remote -v` reports `origin git@github.com:ivan-94/xmind-cli.git` for fetch and push.
- No GitHub Issues were created by this documentation update.

### Open questions / risks

- Existing local `.scratch/` content, if any, may need manual migration or archival before relying entirely on GitHub Issues.
