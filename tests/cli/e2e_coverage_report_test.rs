const REPORT: &str = include_str!("../../docs/technical/e2e-coverage-report.md");
const E2E_PLAN: &str = include_str!("../../docs/technical/e2e-test-plan.md");
const PLAN: &str = include_str!("../../PLAN.md");
const ROOT_NOTES: &str = include_str!("../../implementation-notes.html");

const COMMANDS: &[&str] = &[
    "inspect",
    "sheets",
    "tree",
    "get",
    "find",
    "validate",
    "diff",
    "add",
    "add-tree",
    "set",
    "delete",
    "move",
    "copy",
    "patch",
    "import",
    "export",
    "backup",
    "restore",
    "completion",
];

#[test]
fn e2e_coverage_report_lists_every_documented_command() {
    for command in COMMANDS {
        let command_reference = format!("docs/reference/commands/{command}.md");
        assert!(
            std::path::Path::new(&command_reference).exists(),
            "command reference should exist for {command}"
        );
        assert!(
            REPORT.contains(&format!("| `xmind {command}` |")),
            "coverage report must include a command row for xmind {command}"
        );
    }
}

#[test]
fn e2e_coverage_report_links_from_plan_and_records_blocking_issues() {
    assert!(
        E2E_PLAN.contains("docs/technical/e2e-coverage-report.md"),
        "E2E test plan should link to the living coverage report"
    );

    for issue in ["#3", "#11"] {
        assert!(
            REPORT.contains(issue),
            "coverage report should preserve blocker/follow-up issue {issue}"
        );
    }

    assert!(REPORT.contains("## PR Subset vs Full Matrix Status"));
    assert!(REPORT.contains("## Known Blockers and Follow-ups"));
}

#[test]
fn issue_23_docs_sync_removes_resolved_phase_17_and_issue_15_todos() {
    for stale in [
        "currently returns `invalid_usage`",
        "currently emits no documented envelope",
        "currently performs only parse/read validation",
        "not exposed by clap",
    ] {
        assert!(
            !PLAN.contains(stale),
            "PLAN.md still describes resolved Phase 17 behavior as a gap: {stale}"
        );
    }

    assert!(
        !REPORT.contains("TODO #15"),
        "coverage report should not keep issue #15 as a TODO after batch/exchange/recovery E2E landed"
    );
    assert!(
        !REPORT.contains("Issue #23 should resync")
            && !REPORT.contains("before issue #23 performs final documentation synchronization"),
        "coverage report should not keep issue #23 as a future synchronization blocker"
    );
    assert!(
        !E2E_PLAN.contains("No new test command was executed while writing this planning document"),
        "E2E plan should carry current verification evidence, not the original planning-only note"
    );
    assert!(
        !ROOT_NOTES.contains("无法给出新的本地质量门证据"),
        "implementation notes should not keep the stale local quality-gate blocker after the rustup path is established"
    );
}

#[test]
fn e2e_coverage_report_rows_use_maintainable_test_or_todo_references() {
    for line in REPORT.lines().filter(|line| line.starts_with("| `xmind ")) {
        assert!(
            line.contains("PR subset") || line.contains("Full matrix") || line.contains("TODO #"),
            "coverage rows must expose PR/full/TODO status: {line}"
        );
        assert!(
            line.contains("`tests/") || line.contains("TODO #"),
            "coverage rows must point to a test file/name or an explicit TODO issue: {line}"
        );
        if line.contains("`tests/") {
            assert!(
                line.contains("::"),
                "test references should use tests/path.rs::test_name format: {line}"
            );
        }
    }
}
