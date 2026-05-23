# E2E Coverage Report

Living checklist for GitHub issues #24 and #23. This report maps the command and user-visible branch matrix from `docs/technical/e2e-test-plan.md` and `docs/reference/commands/*.md` to current Rust tests or explicit TODO issues.

## Source Manifest

### Sources

- GitHub issue #24: `https://github.com/ivan-94/xmind-cli/issues/24`.
- GitHub issue #23: `https://github.com/ivan-94/xmind-cli/issues/23`.
- GitHub issue #15: `https://github.com/ivan-94/xmind-cli/issues/15`.
- Parent PRD #1 implementation notes: `docs/prd/1/implementation-notes.html`.
- Repository implementation notes: `implementation-notes.html`.
- E2E plan: `docs/technical/e2e-test-plan.md`.
- Real XMind App fixture manifest: `tests/fixtures/xmind/manifest.md`.
- Command references: `docs/reference/commands/*.md`.
- Current E2E tests: `tests/e2e/pr_subset_test.rs`, `tests/e2e/full_matrix_test.rs`, `tests/e2e/red_contract_gaps_test.rs`, and `tests/e2e/batch_exchange_recovery_test.rs`.
- Current CLI tests: `tests/cli/*_test.rs`.
- Workflow requirements: `~/.agents/docs/agents/workflows.md` and `~/.agents/docs/agents/handoff-policy.md`.

### Produced artifacts

- `docs/technical/e2e-coverage-report.md`.
- `tests/cli/e2e_coverage_report_test.rs`.
- Updated link in `docs/technical/e2e-test-plan.md`.
- Updated notes in `implementation-notes.html` and `docs/prd/1/implementation-notes.html`.

### Key decisions

- Status is reported as `PR subset`, `Full matrix`, or `TODO #...`.
- Rows can cite CLI integration tests when they currently provide the strongest user-visible branch coverage, but full E2E graduation remains tracked by TODO issues.
- Issue #15 has graduated batch, exchange, recovery, and shell-integration branches into the dedicated `e2e_batch_exchange_recovery` target.
- Issue #23 keeps this report synchronized with Phase 17 command closures and issue #15 coverage.

### Verification evidence

- Guarded by `tests/cli/e2e_coverage_report_test.rs`.
- `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test e2e_coverage_report_test --all-features`.

### Open questions / risks

- Full matrix execution now has an initial real XMind App fixture from #11, but richer app-saved variants remain useful after the CLI stabilizes.
- GitHub-required PR E2E enforcement was configured through branch protection setup in #3.
- Batch/exchange/recovery rows now cite `tests/e2e/batch_exchange_recovery_test.rs`; remaining TODOs are real fixture/full-matrix governance rather than issue #15 coverage work.

## PR Subset vs Full Matrix Status

- PR subset: current required runner is `cargo test --test e2e_pr_subset --all-features`, documented by issue #12 and exercised in `tests/e2e/pr_subset_test.rs`.
- Full matrix: inventory exists in `tests/e2e/full_matrix_test.rs::full_matrix_command_inventory_is_filterable_for_nightly_or_release_jobs`, but the test is ignored so release/nightly jobs can opt in explicitly.
- Release smoke: binary jobs intentionally run only `xmind --version`, `xmind tree ... --json`, and `xmind validate ... --json`; the full command matrix remains in Rust E2E tests.
- Real fixture axis: #11 added `tests/fixtures/xmind/real-app/real-app-fixture.xmind`; the full matrix can grow additional app-saved variants later.
- CI enforcement axis: #3 configured branch protection and required checks for the PR subset on GitHub.

## Command Coverage Matrix

| Command | User-visible branches from plan/reference | Current coverage | Status / TODO |
| --- | --- | --- | --- |
| `xmind inspect` | Supported format, resources/capabilities, malformed workbook, unsupported variant, fields/compact output, human output. | `tests/e2e/pr_subset_test.rs::read_e2e_inspect_reports_supported_resources_and_parse_errors`; `tests/cli/inspect_test.rs::inspect_json_reports_unknown_package_entry_preservation`; `tests/cli/inspect_test.rs::inspect_json_reports_unknown_json_field_preservation`; `tests/cli/runtime_errors_test.rs::json_validate_unsupported_workbook_variant_returns_unsupported_format`. | PR subset covered; Full matrix TODO #11 for real-app fixture variants. |
| `xmind sheets` | Duplicate titles, field filtering, sheet metadata, missing/parse errors, human output. | `tests/e2e/pr_subset_test.rs::read_e2e_sheets_covers_duplicates_fields_metadata_and_human_output`; `tests/cli/sheets_test.rs::sheets_json_compact_format_limits_sheet_fields`. | PR subset covered; Full matrix TODO #11 for real-app duplicate-sheet fixtures. |
| `xmind tree` | Depth, fields, include assets, sheet selection, text/human output, sheet errors. | `tests/e2e/pr_subset_test.rs::read_e2e_tree_covers_depth_fields_assets_sheet_selection_and_human_output`; `tests/e2e/pr_subset_test.rs::real_app_fixture_decodes_tree_and_validates`; `tests/e2e/pr_subset_test.rs::default_pr_subset_covers_representative_json_error_families`; `tests/cli/tree_test.rs::tree_json_ambiguous_sheet_title_returns_candidates`. | PR subset covered; Full matrix has #11 app-saved seed and can add deeper variants later. |
| `xmind get` | Id/path/title/query selectors, depth, assets, sheet selection, not found, ambiguous selector, fields/compact output. | `tests/e2e/pr_subset_test.rs::read_e2e_get_covers_selectors_depth_assets_and_selector_errors`; `tests/cli/get_test.rs::get_json_compact_format_limits_topic_fields`. | PR subset covered; Full matrix TODO #11 for path escaping/non-ASCII fixture cases. |
| `xmind find` | Exact title, title contains, content contains, query selectors, limit/offset, no matches, invalid usage. | `tests/e2e/pr_subset_test.rs::read_e2e_find_covers_match_modes_pagination_and_empty_results`; `tests/cli/find_test.rs::find_json_query_and_requires_both_conditions`; `tests/cli/find_test.rs::find_json_query_parentheses_group_expression`; `tests/e2e/pr_subset_test.rs::read_e2e_read_commands_return_invalid_usage_errors`. | PR subset covered; Full matrix TODO #11 for richer metadata fixtures. |
| `xmind validate` | Valid, warnings, `--strict` warning failure, structural errors, malformed workbook, human output. | `tests/e2e/pr_subset_test.rs::read_e2e_validate_covers_valid_strict_parse_error_and_human_output`; `tests/e2e/pr_subset_test.rs::real_app_fixture_decodes_tree_and_validates`; `tests/e2e/pr_subset_test.rs::read_e2e_validate_warnings_are_reported_and_strict_turns_them_into_failures`; `tests/e2e/pr_subset_test.rs::read_e2e_validate_reports_structural_errors`; `tests/e2e/red_contract_gaps_test.rs::validate_strict_reports_structural_diagnostics`. | PR subset covered; Full matrix has #11 app-saved seed and can add private metadata variants later. |
| `xmind diff` | Documented single-workbook summary/changes envelope, no-change human output, file/parse/usage/sheet errors. | `tests/e2e/pr_subset_test.rs::default_pr_subset_covers_read_command_success_paths`; `tests/e2e/pr_subset_test.rs::default_pr_subset_checks_lightweight_human_output`; `tests/e2e/red_contract_gaps_test.rs::diff_json_emits_documented_summary_and_changes_envelope`. | PR subset covered for current contract; Full matrix TODO #11 for real-app workbook variants. |
| `xmind add` | Dry-run, apply, backup, positions, create missing path, parent not found, ambiguous parent. | `tests/e2e/pr_subset_test.rs::default_pr_subset_applies_topic_mutations_to_temp_copies_then_validates`; `tests/e2e/pr_subset_test.rs::e2e_add_covers_positions_missing_path_backup_and_selector_errors`; `tests/cli/add_test.rs::add_dry_run_human_output_includes_outline_diff`. | PR subset covered; Full matrix TODO #11 for real-app mutation fixtures. |
| `xmind add-tree` | YAML/JSON/Markdown input, dry-run/apply parity, backup, invalid tree input, Markdown modes/errors. | `tests/e2e/pr_subset_test.rs::default_pr_subset_covers_batch_exchange_backup_restore_paths`; `tests/e2e/red_contract_gaps_test.rs::add_tree_apply_mutates_copied_workbook`; `tests/e2e/batch_exchange_recovery_test.rs::add_tree_e2e_covers_input_formats_dry_run_apply_backup_and_invalid_input`; `tests/cli/add_tree_test.rs::add_tree_markdown_accepts_all_documented_modes`. | PR subset and batch/recovery E2E covered; Full matrix TODO #11 for real fixtures. |
| `xmind set` | Editable fields, clear fields, image attach/replace/clear, unsupported asset, backup, root behavior. | `tests/e2e/pr_subset_test.rs::default_pr_subset_applies_topic_mutations_to_temp_copies_then_validates`; `tests/e2e/pr_subset_test.rs::e2e_set_covers_editable_fields_clear_image_paths_and_asset_errors`; `tests/cli/set_test.rs::set_clear_repeated_apply_clears_topic_fields`; `tests/cli/set_test.rs::set_image_apply_preserves_unrelated_package_entries`. | PR subset covered; Full matrix TODO #11 for real image/resource fixtures. |
| `xmind delete` | Subtree, children-only, promote-children, root rejection, backup, validation/write errors. | `tests/e2e/pr_subset_test.rs::default_pr_subset_applies_topic_mutations_to_temp_copies_then_validates`; `tests/e2e/pr_subset_test.rs::e2e_delete_covers_subtree_children_only_promote_root_rejection_and_backup`; `tests/cli/delete_test.rs::delete_children_only_rejects_root_operation`. | PR subset covered; Full matrix TODO #11 for larger real trees. |
| `xmind move` | Positions, cycle rejection, root rejection, backup, source fixture integrity. | `tests/e2e/pr_subset_test.rs::default_pr_subset_applies_topic_mutations_to_temp_copies_then_validates`; `tests/e2e/pr_subset_test.rs::e2e_move_covers_positions_cycle_root_rejection_backup_and_source_integrity`; `tests/cli/move_test.rs::move_rejects_destination_inside_source_subtree`. | PR subset covered; Full matrix TODO #11 for wider real trees. |
| `xmind copy` | Default id regeneration, positions, `--preserve-ids` guardrail, root rejection, backup. | `tests/e2e/pr_subset_test.rs::default_pr_subset_applies_topic_mutations_to_temp_copies_then_validates`; `tests/e2e/pr_subset_test.rs::e2e_copy_covers_id_regeneration_positions_guardrails_root_rejection_and_backup`; `tests/cli/copy_test.rs::copy_preserve_ids_rejects_same_workbook_copy`. | PR subset covered; Full matrix TODO #11 for real duplicate-title fixtures. |
| `xmind patch` | Every op, aliases, dry-run/apply parity, operation-indexed errors, rollback, backup. | `tests/e2e/pr_subset_test.rs::default_pr_subset_covers_batch_exchange_backup_restore_paths`; `tests/e2e/red_contract_gaps_test.rs::patch_apply_mutates_copied_workbook`; `tests/e2e/red_contract_gaps_test.rs::patch_apply_rolls_back_when_later_operation_errors`; `tests/e2e/batch_exchange_recovery_test.rs::patch_e2e_covers_every_operation_aliases_parity_errors_rollback_and_backup`. | PR subset and batch/recovery E2E covered; Full matrix TODO #11 for real fixtures. |
| `xmind import` | `--output`/`--into`, YAML/JSON/Markdown input, overwrite, `--into --backup`, no file on dry-run. | `tests/e2e/pr_subset_test.rs::default_pr_subset_covers_batch_exchange_backup_restore_paths`; `tests/e2e/red_contract_gaps_test.rs::import_into_apply_backup_preserves_existing_workbook_safety`; `tests/e2e/batch_exchange_recovery_test.rs::import_export_e2e_covers_formats_output_into_overwrite_assets_and_dry_run_safety`; `tests/cli/import_test.rs::export_markdown_then_import_output_round_trips_topic_outline`. | PR subset and batch/recovery E2E covered; Full matrix TODO #11 for real fixtures. |
| `xmind export` | JSON/Markdown/outline/text/assets, stdout/output, overwrite behavior, selector/asset errors. | `tests/e2e/pr_subset_test.rs::default_pr_subset_covers_batch_exchange_backup_restore_paths`; `tests/e2e/batch_exchange_recovery_test.rs::import_export_e2e_covers_formats_output_into_overwrite_assets_and_dry_run_safety`; `tests/cli/export_test.rs::export_format_assets_output_writes_embedded_resources_to_directory`. | PR subset and batch/recovery E2E covered; Full matrix TODO #11 for real fixtures. |
| `xmind backup` | Default dir, custom dir, JSON output, invalid path/write failure. | `tests/e2e/pr_subset_test.rs::default_pr_subset_covers_batch_exchange_backup_restore_paths`; `tests/e2e/batch_exchange_recovery_test.rs::backup_restore_and_completion_e2e_cover_recovery_and_shell_integration`; `tests/cli/backup_test.rs::backup_json_writes_to_custom_backup_dir`. | PR subset and batch/recovery E2E covered; Full matrix TODO #11 for real fixtures. |
| `xmind restore` | Dry-run/apply, backup-before-restore, latest backup selection, invalid backup. | `tests/e2e/pr_subset_test.rs::default_pr_subset_covers_batch_exchange_backup_restore_paths`; `tests/e2e/batch_exchange_recovery_test.rs::backup_restore_and_completion_e2e_cover_recovery_and_shell_integration`; `tests/cli/restore_test.rs::restore_dry_run_reports_latest_backup_without_writing`. | PR subset and batch/recovery E2E covered; Full matrix TODO #11 for real fixtures. |
| `xmind completion` | Shell variants, no workbook access, non-JSON stdout. | `tests/e2e/pr_subset_test.rs::default_pr_subset_checks_lightweight_human_output`; `tests/e2e/batch_exchange_recovery_test.rs::backup_restore_and_completion_e2e_cover_recovery_and_shell_integration`; `tests/cli/completion_test.rs::completion_bash_prints_shell_completion_script`. | PR subset and batch/recovery E2E covered. |

## Known Blockers and Follow-ups

- #3: Branch protection and required status checks were configured with `scripts/configure-branch-protection.sh apply`.
- #11: One privacy-safe real XMind App-saved golden fixture is committed; future release/nightly expansion can add broader app-saved variants after the CLI stabilizes.
