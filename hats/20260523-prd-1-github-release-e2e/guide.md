# PRD #1 GitHub Release and E2E Program HAT Guide

<!-- HAT:BEGIN metadata -->
## Metadata

- Source: GitHub PRD issue `#1`, child issues `#2` through `#24`, and branch `codex/prd-1-github-release-e2e-program`.
- Created: 2026-05-23
- Updated: 2026-05-23
- Repo root: `/Users/ivan/workspace/ai/xmind-cli`
- Mode: `blank`
- Mode rationale: this is a Rust CLI release/E2E verification path. It does not need an existing database, tenant, account, or attached service state. Verification uses a fresh local checkout, committed fixtures, build artifacts, generated release-contract files, and optional maintainer-controlled GitHub repository settings.
- Preparation status: `syntax-checked`
- Prepare script: `hats/20260523-prd-1-github-release-e2e/prepare.sh`
<!-- HAT:END metadata -->

## Source Manifest

### Sources

- User goal: `/goal [$deliver-prd] #1, 实现所有子 issue`.
- Project agent instructions: `AGENTS.md` content supplied in the thread, including Source Manifest and `implementation-notes.html` maintenance rules.
- Workflow rules: `~/.agents/docs/agents/workflows.md` and `~/.agents/docs/agents/handoff-policy.md`.
- PRD delivery plan and backlog: `PLAN.md`.
- Implementation notes: `implementation-notes.html` and `docs/prd/1/implementation-notes.html`.
- Release and install contracts: `Cargo.toml`, `.github/workflows/release.yml`, `.github/scripts/extract-release-notes.sh`, `scripts/install.sh`, `docs/installation.md`, `docs/technical/release-policy.md`, `CHANGELOG.md`.
- E2E and fixture contracts: `docs/technical/e2e-test-plan.md`, `docs/technical/e2e-coverage-report.md`, `tests/e2e/`, `tests/cli/`, `tests/fixtures/xmind/manifest.md`.
- Cross-review evidence: `.scratch/cross-review/20260523-181334-claude.md` and self-review result in the parent delivery thread.

### Produced Artifacts

- HAT guide: `hats/20260523-prd-1-github-release-e2e/guide.md`.
- HAT prepare script: `hats/20260523-prd-1-github-release-e2e/prepare.sh`.
- Cross-review fix commit: `ed4b37f Fix release archive and notes contracts`.
- Real XMind App fixture: `tests/fixtures/xmind/real-app/real-app-fixture.xmind`.

### Key Decisions

- Issue `#3` branch protection was configured through `scripts/configure-branch-protection.sh apply`.
- Issue `#11` now has one repository-safe real XMind App fixture created through Computer Use; broader app-saved variants remain future full-matrix expansion after the CLI stabilizes.
- Keep Homebrew as a documented future path, not a delivered formula or tap automation; the user explicitly deferred it until the program is stable.
- Fix the cross-review P1 by pinning cargo-dist Unix archives to `.tar.gz`, matching `scripts/install.sh` and release docs.
- Fix the release-note P2 by extracting the matching `CHANGELOG.md` version section for the pushed tag and failing release publication if it is missing.

### Verification Evidence

- `PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh` passed after all slice merges and after the cross-review release fix.
- Cross-review external track completed via `claude -p ... --disallowedTools Edit Write MultiEdit NotebookEdit`; log: `.scratch/cross-review/20260523-181334-claude.md`.
- Cross-review self track reported one P1 and one P2; both were fixed in commit `ed4b37f`.
- Focused worker validation before commit: `cargo fmt --all -- --check`, `cargo test --test release_workflow_test --all-features`, `cargo test --test install_script_test --all-features`, release-policy doc example test, and `cargo check --workspace --all-targets --all-features`.
- Real fixture validation: `xmind tree tests/fixtures/xmind/real-app/real-app-fixture.xmind --json` and `xmind validate tests/fixtures/xmind/real-app/real-app-fixture.xmind --json` both passed through `cargo run --quiet`.
- HAT prepare syntax validation: `bash -n hats/20260523-prd-1-github-release-e2e/prepare.sh`.
- `shellcheck`: not available in the local environment, so it was not run.

### Open Questions / Risks

- GitHub branch protection was configured through the GitHub API and verified with `gh api /repos/ivan-94/xmind-cli/branches/master/protection`.
- Real XMind App fixture coverage has an initial app-saved fixture; deeper/more varied app-saved files can be added later when the program is stable.
- A real tagged cargo-dist release and Homebrew formula publication are intentionally deferred until the program is stable. The workflow now has stronger contract tests, but first release publication should still be monitored by a maintainer.
- Full E2E matrix remains intentionally ignored in default PR tests and should be run explicitly for release/nightly confidence when needed.

## Environment Information

- Execution environment: local macOS checkout.
- Runtime: Rust CLI binary built from source.
- App URL: not applicable.
- Database: not applicable.
- Migration command: not applicable.
- Start command: not applicable.
- Build command: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo build --workspace`.
- Quality gate command: `PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh`.
- HAT prepare command: `bash hats/20260523-prd-1-github-release-e2e/prepare.sh prepare`.
- Cleanup command: `bash hats/20260523-prd-1-github-release-e2e/prepare.sh cleanup`.

## Blockers

| Item | Needed From | Status | Notes |
| --- | --- | --- | --- |
| Branch protection configuration | Repository maintainer/admin | Done | API verification reports strict required checks `Rust quality gate`, `Stable PR E2E subset`, and `Security`; force pushes and deletions are disabled. |
| Real XMind App fixtures | Local XMind App via Computer Use | Done for initial fixture | `tests/fixtures/xmind/real-app/real-app-fixture.xmind` is committed and covered by read/validate checks. |
| First tagged GitHub Release | Maintainer | Not run in HAT | Use release checklist and monitor artifact names/checksums. |

## Acceptance Accounts

No user accounts are required for local CLI HAT. GitHub maintainer credentials are required only for branch protection settings, tag publication, and release review.

## Acceptance Data

- Committed fixtures under `tests/fixtures/xmind/`.
- Synthetic fixture manifest and real-app fixture provenance under `tests/fixtures/xmind/manifest.md`.
- Temporary HAT workspace created by `prepare.sh prepare`: `.scratch/hat/prd-1-github-release-e2e/`.
- Cleanup only removes the HAT workspace above.

## Data Migration Check

No database, schema migration, or external persistent data migration exists for this CLI change set. File mutation safety is verified through transactional writer tests, backup/restore tests, and E2E workflows against copied fixtures.

<!-- HAT:BEGIN checklist -->
## Acceptance Checklist

### P0 - Release and CLI Safety

#### P0.1 Full Quality Gate

Preconditions:
- The checkout is on `codex/prd-1-github-release-e2e-program`.
- Rust commands use `PATH=/opt/homebrew/opt/rustup/bin:$PATH`.

Steps:
1. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh`.
2. Confirm the command exits 0.
3. Confirm the output includes `release_workflow_test`, `install_script_test`, `e2e_pr_subset`, `e2e_red_contract_gaps`, and `e2e_coverage_report_test`.

Expected:
- All default tests pass.
- `e2e_full_matrix` remains ignored with the documented reason.

Evidence:
- Terminal output or CI check URL.

Notes:
- This was already run locally after `ed4b37f` and passed.

#### P0.2 Install Artifact Contract

Preconditions:
- Commit `ed4b37f` or later is present.

Steps:
1. Inspect `Cargo.toml` `[workspace.metadata.dist]`.
2. Inspect `scripts/install.sh`.
3. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test release_workflow_test --all-features`.
4. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test install_script_test --all-features`.

Expected:
- `Cargo.toml` sets `unix-archive = ".tar.gz"`.
- Unix install artifacts are named as `.tar.gz`; Windows remains `.zip`.
- Focused tests pass.

Evidence:
- File references and test output.

#### P0.3 Mutation Safety

Preconditions:
- HAT prepare has built the CLI or `cargo test` can build it.

Steps:
1. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test e2e_red_contract_gaps --all-features`.
2. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test e2e_pr_subset --all-features`.

Expected:
- Patch apply rollback, add-tree apply, import backup safety, validate diagnostics, and representative read/mutation flows pass.

Evidence:
- Test output.

### P1 - Release Operations

#### P1.1 Release Notes Extraction

Preconditions:
- `CHANGELOG.md` has or can be edited to have a version heading for the tag under test.

Steps:
1. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test release_workflow_test release_notes_extraction_uses_matching_version_section_only --all-features -- --exact`.
2. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test release_workflow_test release_notes_extraction_fails_when_version_section_is_missing --all-features -- --exact`.
3. Inspect `.github/workflows/release.yml`.

Expected:
- The workflow runs `.github/scripts/extract-release-notes.sh`.
- GitHub Release uses `body_path: target/release-notes.md`.
- Missing matching changelog sections fail before publication.

Evidence:
- Test output and workflow snippet.

#### P1.2 Branch Protection Human Check

Preconditions:
- PR exists and CI check names are visible in GitHub.
- Maintainer/admin has repository settings access.

Steps:
1. Run `gh api /repos/ivan-94/xmind-cli/branches/master/protection --jq '{strict:.required_status_checks.strict,contexts:.required_status_checks.contexts,required_reviews:.required_pull_request_reviews.required_approving_review_count,allow_force_pushes:.allow_force_pushes.enabled,allow_deletions:.allow_deletions.enabled}'`.
2. Confirm strict checks include `Rust quality gate`, `Stable PR E2E subset`, and `Security`.
3. Confirm `allow_force_pushes` and `allow_deletions` are both `false`.

Expected:
- Branch protection enforces the intended PR gate.

Evidence:
- GitHub API output.

Notes:
- This closes the automated issue `#3` setup path; later PRs still need live CI evidence.

#### P1.3 Real XMind Fixture Coverage

Preconditions:
- `tests/fixtures/xmind/real-app/real-app-fixture.xmind` exists.

Steps:
1. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- tree tests/fixtures/xmind/real-app/real-app-fixture.xmind --json`.
2. Run `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- validate tests/fixtures/xmind/real-app/real-app-fixture.xmind --json`.
3. Confirm `tests/fixtures/xmind/manifest.md` labels the fixture `real-xmind-app` and records privacy/license notes.

Expected:
- The app-saved fixture is documented and passes selected CLI read/validate coverage.

Evidence:
- Fixture manifest row and test output.

Notes:
- Broader app-saved variants can be added later for release/nightly depth after the CLI stabilizes.

### P2 - Documentation and Handoff

#### P2.1 Source Manifests

Steps:
1. Inspect `README.md`, `PLAN.md`, `implementation-notes.html`, `docs/prd/1/implementation-notes.html`, and this HAT guide.
2. Confirm each durable artifact points to original sources and verification evidence rather than only summaries.

Expected:
- Source Manifest sections are present and actionable.

Evidence:
- File references.

#### P2.2 Release Documentation Does Not Overclaim

Steps:
1. Inspect `README.md`, `docs/installation.md`, and `docs/technical/release-policy.md`.
2. Confirm GitHub Release binaries are described for tagged releases, Homebrew remains future/deferred, crates.io is not presented as a first release path, and unsupported Linux arm64 is not advertised.

Expected:
- Documentation matches current support state.

Evidence:
- File references.
<!-- HAT:END checklist -->

## Execution Method

Human verifier:
- Use terminal commands in this guide.
- Use GitHub UI only for branch protection, PR review, CI evidence, and release publication.
- For any future real fixtures, verify `.xmind` provenance, privacy, size, and CLI read/validate behavior before committing them.

Agent notes:
- This repository has no browser UI or `window.__hat` surface.
- Prefer structured command output and tests over screenshots.
- Do not run destructive cleanup outside `.scratch/hat/prd-1-github-release-e2e/`.

## Pass Criteria

- All P0 checks pass.
- P1 checks either pass or are explicitly marked deferred with evidence and owner.
- No release documentation claims a channel or platform that is not implemented.
- Cleanup strategy is limited to HAT-created `.scratch/hat/prd-1-github-release-e2e/`.

<!-- HAT:MANUAL notes -->
## Manual Execution Record

| Time | Executor | Scenario | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- |
| TODO | TODO | TODO | TODO | TODO | TODO |
<!-- HAT:ENDMANUAL notes -->
