use assert_cmd::Command;
use serde_json::Value;

#[test]
fn set_title_dry_run_reports_updated_topic_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "id:topic-payment",
            "--title",
            "Payments",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["updated"]["id"], "topic-payment");
    assert_eq!(body["result"]["updated"]["old_path"], "/Q2/Payment");
    assert_eq!(body["result"]["updated"]["new_path"], "/Q2/Payments");
    assert_eq!(body["result"]["updated"]["changed_fields"][0], "title");
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payments");

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
