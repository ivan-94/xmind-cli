#[path = "support.rs"]
#[allow(dead_code)]
mod support;

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;
use support::{copy_fixture, temp_file, MINIMAL_FIXTURE};
use zip::write::FileOptions;

fn command_output(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("xmind")
        .expect("xmind binary is built for red E2E tests")
        .args(args)
        .output()
        .expect("xmind command runs")
}

fn expect_json_success(args: &[&str]) -> Value {
    let output = command_output(args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected command to satisfy documented contract: xmind {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is the documented JSON envelope")
}

fn expect_json_error(args: &[&str], exit_code: i32) -> Value {
    let output = command_output(args);
    assert_eq!(
        output.status.code(),
        Some(exit_code),
        "expected command to fail with documented diagnostics: xmind {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("error stdout is the documented JSON envelope")
}

fn tree_titles(workbook: &str) -> Vec<String> {
    let body = expect_json_success(&["tree", workbook, "--json", "--depth", "3"]);
    let mut titles = Vec::new();
    collect_titles(&body["result"]["root"], &mut titles);
    titles
}

fn collect_titles(topic: &Value, titles: &mut Vec<String>) {
    if let Some(title) = topic["title"].as_str() {
        titles.push(title.to_owned());
    }
    if let Some(children) = topic["children"].as_array() {
        for child in children {
            collect_titles(child, titles);
        }
    }
}

fn write_duplicate_topic_id_fixture(path: &Path) {
    let file = fs::File::create(path).expect("synthetic structural fixture is created");
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file("content.json", FileOptions::default())
        .expect("content.json entry is created");
    zip.write_all(
        br#"[
  {
    "id": "sheet-roadmap",
    "title": "Roadmap",
    "rootTopic": {
      "id": "topic-root",
      "title": "Roadmap",
      "children": {
        "attached": [
          {
            "id": "topic-duplicate",
            "title": "Q2",
            "children": {
              "attached": [
                {
                  "id": "topic-duplicate",
                  "title": "Payment"
                }
              ]
            }
          }
        ]
      }
    }
  }
]"#,
    )
    .expect("content.json fixture is written");
    zip.finish()
        .expect("synthetic structural fixture is finalized");
}

#[test]
#[ignore = "red contract test for PRD #1 issue #17; enable while implementing issue #18"]
fn add_tree_apply_mutates_copied_workbook() {
    let fixture = copy_fixture(MINIMAL_FIXTURE, "add-tree-apply.xmind");
    let workbook = fixture.path_arg();

    let body = expect_json_success(&[
        "add-tree",
        &workbook,
        "--parent",
        "path:/Q2",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--apply",
        "--json",
    ]);

    assert_eq!(body["command"], "add-tree");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["summary"]["added"], 9);
    assert!(tree_titles(&workbook).contains(&"支付能力".to_owned()));
    fixture.assert_source_unchanged();
}

#[test]
#[ignore = "red contract test for PRD #1 issue #17; enable while implementing issue #19"]
fn patch_apply_mutates_copied_workbook() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for patch input");
    let ops = temp_file(
        temp_dir.path(),
        "patch-apply.yaml",
        "ops:\n  - op: add\n    parent: path:/Q2\n    title: Refund\n",
    );
    let fixture = copy_fixture(MINIMAL_FIXTURE, "patch-apply.xmind");
    let workbook = fixture.path_arg();

    let body = expect_json_success(&[
        "patch", &workbook, "--ops", &ops, "--apply", "--backup", "--json",
    ]);

    assert_eq!(body["command"], "patch");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["operations"][0]["status"], "applied");
    assert_eq!(body["result"]["summary"]["added"], 1);
    assert!(body["result"]["backup_path"].as_str().is_some());
    assert!(tree_titles(&workbook).contains(&"Refund".to_owned()));
    fixture.assert_source_unchanged();
}

#[test]
#[ignore = "red contract test for PRD #1 issue #17; enable while implementing issue #19"]
fn patch_apply_rolls_back_when_later_operation_errors() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for patch input");
    let ops = temp_file(
        temp_dir.path(),
        "patch-rollback.yaml",
        "ops:\n  - op: add\n    parent: path:/Q2\n    title: Should Roll Back\n  - op: set\n    node: path:/missing\n    fields:\n      title: Unreachable\n",
    );
    let fixture = copy_fixture(MINIMAL_FIXTURE, "patch-rollback.xmind");
    let workbook = fixture.path_arg();

    let body = expect_json_error(&["patch", &workbook, "--ops", &ops, "--apply", "--json"], 5);

    assert_eq!(body["command"], "patch");
    assert_eq!(body["error"]["code"], "not_found");
    assert_eq!(body["error"]["operation_index"], 1);
    assert!(
        !tree_titles(&workbook).contains(&"Should Roll Back".to_owned()),
        "patch --apply must leave the original workbook untouched after a later operation fails"
    );
    fixture.assert_source_unchanged();
}

#[test]
fn diff_json_emits_documented_summary_and_changes_envelope() {
    let body = expect_json_success(&["diff", MINIMAL_FIXTURE, "--json"]);

    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "diff");
    assert_eq!(body["workbook"], MINIMAL_FIXTURE);
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert!(body["result"]["summary"]["added"].is_number());
    assert!(body["result"]["summary"]["updated"].is_number());
    assert!(body["result"]["summary"]["deleted"].is_number());
    assert!(body["result"]["summary"]["moved"].is_number());
    assert!(body["result"]["changes"].is_array());
}

#[test]
fn validate_strict_reports_structural_diagnostics() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for structural fixture");
    let workbook = temp_dir.path().join("duplicate-topic-id.xmind");
    write_duplicate_topic_id_fixture(&workbook);
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let body = expect_json_error(&["validate", &workbook_arg, "--strict", "--json"], 9);

    assert_eq!(body["command"], "validate");
    assert_eq!(body["error"]["code"], "validation_failed");
    assert_eq!(body["result"]["valid"], false);
    let errors = body["result"]["errors"]
        .as_array()
        .expect("validation diagnostics include structural errors");
    assert!(
        errors.iter().any(|error| {
            error["code"] == "duplicate_topic_id"
                && error["path"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("topic-duplicate")
        }),
        "validate --strict must identify duplicate topic ids with stable structural diagnostics"
    );
}

#[test]
fn import_into_apply_backup_preserves_existing_workbook_safety() {
    let fixture = copy_fixture(MINIMAL_FIXTURE, "import-into-backup.xmind");
    let workbook = fixture.path_arg();

    let body = expect_json_success(&[
        "import",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--into",
        &workbook,
        "--parent",
        "path:/Q2",
        "--apply",
        "--backup",
        "--json",
    ]);

    assert_eq!(body["command"], "import");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/支付能力");
    let backup_path = body["result"]["backup_path"]
        .as_str()
        .expect("import --into --backup returns the created backup path");
    assert!(Path::new(backup_path).exists());
    fixture.assert_backup_matches_original(&body);
    assert!(tree_titles(&workbook).contains(&"支付能力".to_owned()));
    fixture.assert_source_unchanged();
}
