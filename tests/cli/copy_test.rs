use assert_cmd::Command;
use serde_json::Value;
use std::fs;

#[test]
fn copy_apply_copies_topic_to_destination_parent_with_new_id() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-copy.xmind");
    fs::copy("tests/fixtures/xmind/duplicate-titles.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "copy",
            &workbook_arg,
            "--node",
            "id:topic-payment-q1",
            "--to",
            "root",
            "--apply",
            "--json",
        ])
        .output()
        .expect("copy command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json copy output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "copy");
    assert_eq!(body["applied"], true);
    assert_eq!(
        body["result"]["copied_root"]["source_id"],
        "topic-payment-q1"
    );
    assert_eq!(
        body["result"]["copied_root"]["new_id"],
        "topic-payment-q1-copy"
    );
    assert_eq!(body["result"]["copied_root"]["path"], "/Payment");
    assert_eq!(body["result"]["summary"]["added"], 1);

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "2"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    let root_children = tree["result"]["root"]["children"]
        .as_array()
        .expect("root children is an array");
    assert_eq!(root_children[0]["children"][0]["id"], "topic-payment-q1");
    assert_eq!(root_children[2]["id"], "topic-payment-q1-copy");
    assert_eq!(root_children[2]["title"], "Payment");
    assert_eq!(root_children[2]["path"], "/Payment");
}

#[test]
fn copy_preserve_ids_rejects_same_workbook_copy() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "copy",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--node",
            "id:topic-payment-q1",
            "--to",
            "root",
            "--preserve-ids",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("copy command runs");

    assert_eq!(output.status.code(), Some(8));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "patch_conflict");
    assert_eq!(
        body["error"]["suggested_fix"],
        "Omit --preserve-ids when copying within the same workbook."
    );
}

#[test]
fn copy_position_first_inserts_copy_as_first_destination_child() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("copy-position-first.xmind");
    fs::copy("tests/fixtures/xmind/duplicate-titles.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "copy",
            &workbook_arg,
            "--node",
            "id:topic-payment-q1",
            "--to",
            "root",
            "--position",
            "first",
            "--apply",
            "--json",
        ])
        .output()
        .expect("copy command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["result"]["copied_root"]["path"], "/Payment");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "1"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(
        tree["result"]["root"]["children"][0]["id"],
        "topic-payment-q1-copy"
    );
}

#[test]
fn copy_position_last_inserts_copy_as_last_destination_child() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("copy-position-last.xmind");
    fs::copy("tests/fixtures/xmind/duplicate-titles.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "copy",
            &workbook_arg,
            "--node",
            "id:topic-payment-q1",
            "--to",
            "root",
            "--position",
            "last",
            "--apply",
            "--json",
        ])
        .output()
        .expect("copy command runs");

    assert_eq!(output.status.code(), Some(0));
    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "1"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(
        tree["result"]["root"]["children"][2]["id"],
        "topic-payment-q1-copy"
    );
}

#[test]
fn copy_position_index_inserts_copy_at_zero_based_index() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("copy-position-index.xmind");
    fs::copy("tests/fixtures/xmind/duplicate-titles.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "copy",
            &workbook_arg,
            "--node",
            "id:topic-payment-q1",
            "--to",
            "root",
            "--position",
            "index:1",
            "--apply",
            "--json",
        ])
        .output()
        .expect("copy command runs");

    assert_eq!(output.status.code(), Some(0));
    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "1"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(tree["result"]["root"]["children"][0]["id"], "topic-q1");
    assert_eq!(
        tree["result"]["root"]["children"][1]["id"],
        "topic-payment-q1-copy"
    );
    assert_eq!(tree["result"]["root"]["children"][2]["id"], "topic-q2");
}
