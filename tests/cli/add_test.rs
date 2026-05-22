use assert_cmd::Command;
use serde_json::Value;
use std::fs;

#[test]
fn add_dry_run_reports_created_topic_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--title",
            "Refund",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json add output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "add");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["will_change"], true);
    assert_eq!(body["result"]["parent"]["id"], "topic-q2");
    assert_eq!(body["result"]["parent"]["path"], "/Q2");
    assert_eq!(body["result"]["created"]["path"], "/Q2/Refund");
    assert_eq!(body["result"]["summary"]["added"], 1);
    assert_eq!(body["result"]["diff"][0]["event"], "added");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Refund");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--json",
            "--depth",
            "2",
        ])
        .output()
        .expect("tree command runs after dry run");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(
        tree["result"]["root"]["children"][0]["children"][0]["title"],
        "Payment"
    );
    assert_eq!(
        tree["result"]["root"]["children"][0]["children"]
            .as_array()
            .expect("children is an array")
            .len(),
        1
    );
}

#[test]
fn add_dry_run_human_output_includes_outline_diff() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--title",
            "Refund",
            "--dry-run",
        ])
        .output()
        .expect("add command runs");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "+ /Q2/Refund\n");
}

#[test]
fn add_apply_writes_created_topic_transactionally() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-add.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add",
            &workbook_arg,
            "--parent",
            "path:/Q2",
            "--title",
            "Refund",
            "--apply",
            "--json",
        ])
        .output()
        .expect("add command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json add output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "add");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["created"]["path"], "/Q2/Refund");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "2"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    let children = tree["result"]["root"]["children"][0]["children"]
        .as_array()
        .expect("children is an array");
    assert_eq!(children.len(), 2);
    assert_eq!(children[0]["title"], "Payment");
    assert_eq!(children[1]["title"], "Refund");
}
