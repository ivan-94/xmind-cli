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
- Preserve original sources in GitHub Issue bodies under `Source Manifest`; issues are downstream-agent artifacts.
- Do not add Source Manifest sections to plan-derived or ordinary documentation artifacts unless they are PRDs, issues, HAT artifacts, or explicit handoff documents.
- Human-facing README files, overview pages, product docs, command references, and implementation notes should not receive Source Manifest sections unless intentionally serving as a PRD, issue, HAT artifact, or explicit handoff document.
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
