use assert_cmd::Command;
use serde_json::Value;

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
