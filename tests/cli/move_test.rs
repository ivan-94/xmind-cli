use assert_cmd::Command;
use serde_json::Value;
use std::fs;

#[test]
fn move_dry_run_reports_moved_topic_diff_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "move",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--node",
            "id:topic-payment-q1",
            "--to",
            "root",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("move command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json move output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "move");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["will_change"], true);
    assert_eq!(body["result"]["moved"]["id"], "topic-payment-q1");
    assert_eq!(body["result"]["moved"]["from_path"], "/Q1/Payment");
    assert_eq!(body["result"]["moved"]["to_path"], "/Payment");
    assert_eq!(body["result"]["summary"]["moved"], 1);
    assert_eq!(body["result"]["diff"][0]["event"], "moved");
    assert_eq!(body["result"]["diff"][0]["from"], "/Q1/Payment");
    assert_eq!(body["result"]["diff"][0]["to"], "/Payment");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/duplicate-titles.xmind",
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
fn move_apply_moves_topic_to_destination_parent() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-move.xmind");
    fs::copy("tests/fixtures/xmind/duplicate-titles.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "move",
            &workbook_arg,
            "--node",
            "id:topic-payment-q1",
            "--to",
            "root",
            "--apply",
            "--json",
        ])
        .output()
        .expect("move command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json move output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "move");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["moved"]["from_path"], "/Q1/Payment");
    assert_eq!(body["result"]["moved"]["to_path"], "/Payment");
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
    assert_eq!(root_children[0]["title"], "Q1");
    assert_eq!(
        root_children[0]["children"]
            .as_array()
            .expect("Q1 children is an array")
            .len(),
        0
    );
    assert_eq!(root_children[2]["title"], "Payment");
    assert_eq!(root_children[2]["path"], "/Payment");
}

#[test]
fn move_rejects_destination_inside_source_subtree() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "move",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--node",
            "id:topic-q1",
            "--to",
            "id:topic-payment-q1",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("move command runs");

    assert_eq!(output.status.code(), Some(8));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "patch_conflict");
    assert_eq!(
        body["error"]["suggested_fix"],
        "Choose a destination outside the source subtree."
    );
}
