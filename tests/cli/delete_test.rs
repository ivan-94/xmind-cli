use assert_cmd::Command;
use serde_json::Value;
use std::fs;

#[test]
fn delete_dry_run_reports_deleted_topic_diff_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "delete",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "path:/Q2/Payment",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("delete command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json delete output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "delete");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["will_change"], true);
    assert_eq!(body["result"]["deleted"][0], "/Q2/Payment");
    assert_eq!(body["result"]["summary"]["deleted"], 1);
    assert_eq!(body["result"]["diff"][0]["event"], "deleted");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");

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
}

#[test]
fn delete_apply_removes_topic_subtree() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-delete.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "delete",
            &workbook_arg,
            "--node",
            "id:topic-q2",
            "--apply",
            "--json",
        ])
        .output()
        .expect("delete command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json delete output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "delete");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["deleted"][0], "/Q2");
    assert_eq!(body["result"]["deleted"][1], "/Q2/Payment");
    assert_eq!(body["result"]["summary"]["deleted"], 2);

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "2"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(
        tree["result"]["root"]["children"]
            .as_array()
            .expect("children is an array")
            .len(),
        0
    );
}

#[test]
fn delete_children_only_apply_removes_descendants_but_keeps_topic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-delete-children-only.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "delete",
            &workbook_arg,
            "--node",
            "id:topic-q2",
            "--children-only",
            "--apply",
            "--json",
        ])
        .output()
        .expect("delete command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json delete output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "delete");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["deleted"][0], "/Q2/Payment");
    assert_eq!(body["result"]["summary"]["deleted"], 1);

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "2"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    let q2 = &tree["result"]["root"]["children"][0];
    assert_eq!(q2["title"], "Q2");
    assert_eq!(
        q2["children"]
            .as_array()
            .expect("children is an array")
            .len(),
        0
    );
}

#[test]
fn delete_children_only_rejects_root_operation() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "delete",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "root",
            "--children-only",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("delete command runs");

    assert_eq!(output.status.code(), Some(8));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "root_operation_not_allowed");
    assert_eq!(body["error"]["selector"], "root");
}

#[test]
fn delete_promote_children_apply_removes_topic_and_promotes_children() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-delete-promote-children.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "delete",
            &workbook_arg,
            "--node",
            "id:topic-q2",
            "--promote-children",
            "--apply",
            "--json",
        ])
        .output()
        .expect("delete command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json delete output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "delete");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["deleted"][0], "/Q2");
    assert_eq!(body["result"]["promoted"][0]["from_path"], "/Q2/Payment");
    assert_eq!(body["result"]["promoted"][0]["to_path"], "/Payment");
    assert_eq!(body["result"]["summary"]["deleted"], 1);
    assert_eq!(body["result"]["summary"]["moved"], 1);

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "2"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    let root_children = tree["result"]["root"]["children"]
        .as_array()
        .expect("root children is an array");
    assert_eq!(root_children.len(), 1);
    assert_eq!(root_children[0]["title"], "Payment");
    assert_eq!(root_children[0]["path"], "/Payment");
}
