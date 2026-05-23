# PRD #1 Issue #17 Red Contract E2E Notes

## Source Manifest

### Sources

- GitHub issue #17: `https://github.com/ivan-94/xmind-cli/issues/17`
- Parent PRD #1: `https://github.com/ivan-94/xmind-cli/issues/1`
- `AGENTS.md`
- `~/.agents/docs/agents/workflows.md`
- `~/.agents/docs/agents/handoff-policy.md`
- `PLAN.md` Phase 17
- `implementation-notes.html`
- `docs/prd/1/implementation-notes.html`
- `docs/technical/e2e-test-plan.md`
- `docs/reference/commands/add-tree.md`
- `docs/reference/commands/patch.md`
- `docs/reference/commands/diff.md`
- `docs/reference/commands/validate.md`
- `docs/reference/commands/import.md`
- `docs/reference/mutation-semantics.md`
- `docs/reference/output-formats.md`
- `tests/e2e/support.rs`
- `tests/e2e/pr_subset_test.rs`
- `tests/fixtures/xmind/manifest.md`

### Produced artifacts

- `tests/e2e/red_contract_gaps_test.rs`
- `Cargo.toml`
- `docs/notes/prd-1-issue-17-red-contract-e2e.md`
- `docs/prd/1/implementation-notes.html`

### Key decisions

- Red coverage is isolated in the `e2e_red_contract_gaps` integration-test target.
- Every red test is marked `#[ignore]`, so ordinary `cargo test` and the stable PR E2E subset compile the target but do not run these known failures.
- Later implementation slices should remove `#[ignore]` one test at a time, or run the ignored target directly while working on issues #18 through #22.
- The validate structural-diagnostics case uses a synthetic duplicate-topic-id workbook created inside the test because this is an intentionally invalid structural state.

### Verification evidence

- Intended red command:

  ```bash
  PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test e2e_red_contract_gaps --all-features -- --ignored --nocapture
  ```

  Result on 2026-05-23: failed as intended with 0 passed and 6 failed. Failure reasons matched the known gaps: `add-tree --apply` returned `invalid_usage`, both `patch --apply` tests returned `invalid_usage`, `diff --json` emitted empty stdout instead of a JSON envelope, `validate --strict` returned `ok: true` for duplicate topic ids, and `import --into --backup` rejected `--backup` as an unknown argument.

- Default green subset command:

  ```bash
  PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test e2e_pr_subset --all-features
  ```

  Result on 2026-05-23: passed with 5 passed, 0 failed.

- Gating check:

  ```bash
  PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test e2e_red_contract_gaps --all-features
  ```

  Result on 2026-05-23: passed with 0 run and 6 ignored.

- Full quality gate:

  ```bash
  PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh
  ```

  Result on 2026-05-23: passed. This covers `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-features`.

### Open questions / risks

- `diff` still needs issue #20 to settle and implement the final input semantics behind the documented single-workbook surface.
- `validate` diagnostics names such as `duplicate_topic_id` are asserted as the desired stable contract; issue #21 may adjust only with a corresponding command-reference update.
- Current valid `.xmind` fixtures remain `synthetic-generated`, not `real-xmind-app`; this red slice does not add new golden workbook fixtures.
