# PRD Issues #26-#30 README and CLI Help Polish HAT Guide

<!-- HAT:BEGIN metadata -->
## Metadata

- Source: GitHub issues `#26`, `#27`, `#28`, `#29`, and `#30`, plus the user request in the delivery thread.
- Created: 2026-05-25
- Updated: 2026-05-25
- Repo root: `/Users/ivan/workspace/ai/xmind-cli`
- Mode: `blank`
- Mode rationale: this change set covers repository documentation and local Rust CLI help behavior. It does not need an existing database, account, tenant, browser session, or external service.
- Preparation status: `prepared`
- Prepare script: `hats/20260525-prd-26-30-readme-help-polish/prepare.sh`
<!-- HAT:END metadata -->

## Source Manifest

### Sources

- User request: optimize README normalization, bilingual README support, remove Source Manifest from human-facing docs, improve empty `xmind` behavior, enrich CLI help descriptions and examples, make docs references clickable, and codify Source Manifest scope in `AGENTS.md`.
- User decision: Source Manifest is used only for PRD, issue, HAT, and explicit handoff artifacts in this repository.
- Formal slices: GitHub issues `#26` through `#30`.
- Delivery branch: `codex/prd-26-30-readme-help-polish`.
- Agent workflow rules: `~/.agents/docs/agents/workflows.md` and `~/.agents/docs/agents/handoff-policy.md`.
- Repository instructions: `AGENTS.md`.
- Implementation notes: `docs/prd/26-30/implementation-notes.html`.
- Public docs touched by the delivery: `README.md`, `README.zh-CN.md`, `docs/README.md`, `docs/reference/cli-overview.md`, `docs/technical/README.md`, and `docs/examples/README.md`.
- CLI code and tests touched by the delivery: `src/main.rs`, `src/cli.rs`, `tests/cli/help_snapshots_test.rs`, `tests/cli/invalid_usage_test.rs`, `tests/cli/doc_examples_test.rs`, and snapshot fixtures under `tests/snapshots/`.
- Cross-review evidence: external Claude review log `.scratch/cross-review/20260525-113047-claude.md` and self-review result in the parent delivery thread.

### Produced Artifacts

- HAT guide: `hats/20260525-prd-26-30-readme-help-polish/guide.md`.
- HAT prepare script: `hats/20260525-prd-26-30-readme-help-polish/prepare.sh`.
- Delivery notes: `docs/prd/26-30/implementation-notes.html`.
- Draft PR: to be created after this HAT prepare step.

### Key Decisions

- README files are human-facing public entrypoints, so they do not contain Source Manifest sections.
- `README.md` remains the English default; `README.zh-CN.md` provides the Chinese version and both files cross-link near the top.
- Install documentation remains conservative for an early-stage CLI: Cargo, install script, and manual release assets are documented now; Homebrew is described only as a future channel.
- Empty `xmind` and `xmind --json` invocations now show top-level help and exit successfully instead of rendering blank output.
- CLI help stays English-only for now because the existing CLI contract, tests, and snapshots are English.
- Existing legacy Source Manifest sections outside the README/overview cleanup surface are recorded as a residual P2 risk rather than expanded into this slice.

### Verification Evidence

- `PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh` passed after all slice merges and cross-review follow-up fixes.
- Focused checks passed during delivery: `cargo test --test help_snapshots_test`, `cargo test --test invalid_usage_test`, `cargo test --test doc_examples_test`, `cargo test --test release_workflow_test`, `cargo fmt --all -- --check`, and `git diff --check`.
- External cross-review reported one P2: missing dedicated `--version` test. It was fixed by adding `version_matches_package_version`.
- Self-review reported one P2 documentation overclaim for `diff`. It was fixed by aligning `docs/reference/cli-overview.md` and adding a doc example guard.
- Self-review also reported one P2 residual risk: broad legacy Source Manifest cleanup remains outside this confirmed scope.
- HAT prepare syntax validation: `bash -n hats/20260525-prd-26-30-readme-help-polish/prepare.sh`.
- HAT prepare execution: `PATH=/opt/homebrew/opt/rustup/bin:$PATH bash hats/20260525-prd-26-30-readme-help-polish/prepare.sh prepare` completed with `status=prepared`.
- `shellcheck`: not available in the local environment, so it was not run.

### Open Questions / Risks

- Legacy Source Manifest sections may still exist in older docs that were not part of `README.md`, `docs/README.md`, or overview cleanup. This is intentionally left for a future narrow cleanup if desired.
- The HAT prepare script does not run the full quality gate by default; reviewers should run it explicitly for release-level confidence.
- CLI help wording is snapshot-tested, but final wording acceptance is still a human documentation judgment.

## Environment Information

- Execution environment: local macOS checkout.
- Runtime: Rust CLI binary built from source.
- App URL: not applicable.
- Database: not applicable.
- Migration command: not applicable.
- Start command: not applicable.
- Build command: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo build --workspace`.
- Quality gate command: `PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh`.
- HAT prepare command: `bash hats/20260525-prd-26-30-readme-help-polish/prepare.sh prepare`.
- Cleanup command: `bash hats/20260525-prd-26-30-readme-help-polish/prepare.sh cleanup`.

## Blockers

| Item | Needed From | Status | Notes |
| --- | --- | --- | --- |
| Rust toolchain | Local environment | Available locally | Use `PATH=/opt/homebrew/opt/rustup/bin:$PATH` on this machine. |
| External services | None | Not needed | This is docs and CLI behavior only. |
| Human wording judgment | Reviewer | Pending | Validate README/help phrasing reads like a public open-source project. |

## Acceptance Accounts

No accounts are required for local CLI HAT. GitHub access is only required to review the Draft PR and linked issues.

## Acceptance Data

- Committed repository docs.
- Snapshot fixtures under `tests/snapshots/`.
- Temporary HAT workspace created by `prepare.sh prepare`: `.scratch/hat/prd-26-30-readme-help-polish/`.
- Cleanup only removes the HAT workspace above.

## Data Migration Check

No database, schema migration, or external persistent data migration exists for this change set.

<!-- HAT:BEGIN checklist -->
## Acceptance Checklist

### P0 - Public Docs and CLI Behavior

#### P0.1 README Entrypoints

Preconditions:
- Checkout is on `codex/prd-26-30-readme-help-polish`.

Steps:
1. Open `README.md`.
2. Open `README.zh-CN.md`.
3. Confirm both files cross-link near the top.
4. Confirm neither README contains a `Source Manifest` section.
5. Confirm install guidance documents current channels without presenting Homebrew as a live install path.

Expected:
- English and Chinese README entrypoints exist and read as public documentation.
- Source Manifest is absent from human-facing README content.
- Early-release install guidance is conservative and accurate.

Evidence:
- File links or screenshots.

#### P0.2 Empty CLI Invocation Help

Preconditions:
- Run `bash hats/20260525-prd-26-30-readme-help-polish/prepare.sh prepare`, or build the CLI with `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo build --workspace`.

Steps:
1. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet --`.
2. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- --json`.
3. Confirm both commands print top-level help instead of blank output.
4. Confirm both commands exit 0.

Expected:
- Empty invocations are useful and show help.
- `--json` does not suppress the help fallback into a blank response.

Evidence:
- Terminal output.

#### P0.3 Help Descriptions and Examples

Preconditions:
- CLI builds successfully.

Steps:
1. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- --help`.
2. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- set --help`.
3. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- diff --help`.
4. Confirm the help text contains command descriptions and examples.

Expected:
- Top-level and subcommand help is descriptive enough for a first-time user.
- Examples are concrete and copyable.
- `diff` help and docs describe the current single-workbook diff behavior.

Evidence:
- Terminal output.

### P1 - Documentation Navigation and Agent Rules

#### P1.1 Clickable Docs References

Preconditions:
- Checkout includes merged issues `#26` through `#30`.

Steps:
1. Inspect `docs/README.md`.
2. Inspect `docs/reference/cli-overview.md`.
3. Inspect `docs/technical/README.md`.
4. Inspect `docs/examples/README.md`.
5. Confirm references to other repository files use Markdown links where the target exists.

Expected:
- Human readers can click through between overview, reference, technical, and example docs.
- Link text is readable and does not rely on raw path dumps.

Evidence:
- File links or rendered Markdown screenshots.

#### P1.2 Source Manifest Scope Rule

Preconditions:
- Checkout includes merged issue `#30`.

Steps:
1. Inspect `AGENTS.md`.
2. Confirm Source Manifest is required only for PRDs, issues, HAT artifacts, and explicit handoff documents.
3. Confirm `AGENTS.md` does not require Source Manifest for README, overview pages, product docs, command references, or implementation notes.

Expected:
- Future agents have a durable repo-level rule that matches the user's decision.

Evidence:
- `AGENTS.md` snippet.

### P2 - Regression and Reviewer Confidence

#### P2.1 Focused Test Suite

Steps:
1. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test help_snapshots_test`.
2. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test invalid_usage_test`.
3. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test doc_examples_test`.
4. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test release_workflow_test`.

Expected:
- All focused tests pass.

Evidence:
- Terminal output.

#### P2.2 Full Quality Gate

Steps:
1. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh`.
2. Confirm the command exits 0.

Expected:
- Formatting, linting, and the default test suite pass.
- Any intentionally ignored tests remain explicitly ignored.

Evidence:
- Terminal output or CI check URL.
<!-- HAT:END checklist -->

## Acceptance Execution

- Primary entry: local terminal in the repository root.
- Main tools: `cargo`, `git`, and a Markdown renderer or GitHub PR view.
- Agent notes: this is not a browser application and does not expose `window.__hat`; use command output and rendered Markdown inspection.
- Human judgment required: final wording quality, README readability, and whether examples match the intended open-source tone.

## Pass Criteria

- All P0 checks pass.
- P1 checks have no blocker that would confuse a normal reader or future agent.
- P2 checks have no unexplained failures.
- Known residual legacy Source Manifest cleanup is accepted as outside this PRD slice or converted into a future issue.

<!-- HAT:MANUAL notes -->
## Manual Notes

Add reviewer notes here during human acceptance.
<!-- HAT:ENDMANUAL notes -->

## Execution Record Template

| Time | Executor | Scenario | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- |
| TODO | TODO | TODO | TODO | TODO | TODO |
