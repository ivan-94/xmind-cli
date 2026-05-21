# Issue tracker: Local Markdown

Issues and PRDs for this repo live as markdown files in `.scratch/`.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`
- The PRD is `.scratch/<feature-slug>/PRD.md`
- Implementation issues are `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered from `01`
- Triage state is recorded as a `Status:` line near the top of each issue file (see `triage-labels.md` for the role strings)
- Comments and conversation history append to the bottom of the file under a `## Comments` heading

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/<feature-slug>/` (creating the directory if needed).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user will normally pass the path or the issue number directly.

## Source Manifest

### Sources

- User selection in this setup session: Local markdown issue tracker.
- Skill template: `/Users/ivan/.agents/skills/setup-matt-pocock-skills/issue-tracker-local.md`.
- Repository exploration on 2026-05-21: no git remote and no existing `.scratch/` convention.

### Produced artifacts

- `docs/agents/issue-tracker.md`
- `AGENTS.md`

### Key decisions

- Use `.scratch/<feature-slug>/` as the local task workspace for PRDs and issues.

### Verification evidence

- `git remote -v` returned no remotes.
- `.git/config` had no remote section.

### Open questions / risks

- If this repo later moves to GitHub or GitLab Issues, rerun the setup skill or update this file.
