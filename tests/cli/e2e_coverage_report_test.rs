const REPORT: &str = include_str!("../../docs/technical/e2e-coverage-report.md");
const E2E_PLAN: &str = include_str!("../../docs/technical/e2e-test-plan.md");

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

    for issue in ["#3", "#11", "#15", "#23"] {
        assert!(
            REPORT.contains(issue),
            "coverage report should preserve blocker/follow-up issue {issue}"
        );
    }

    assert!(REPORT.contains("## PR Subset vs Full Matrix Status"));
    assert!(REPORT.contains("## Known Blockers and Follow-ups"));
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
