use assert_cmd::Command;
use serde_json::Value;

#[test]
fn sheets_json_lists_minimal_workbook_sheet() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["sheets", "tests/fixtures/xmind/minimal.xmind", "--json"])
        .output()
        .expect("sheets command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json sheets output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "sheets");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert_eq!(body["warnings"], Value::Array(vec![]));
    assert_eq!(body["result"]["sheets"][0]["id"], "sheet-roadmap");
    assert_eq!(body["result"]["sheets"][0]["index"], 0);
    assert_eq!(body["result"]["sheets"][0]["title"], "Roadmap");
    assert_eq!(body["result"]["sheets"][0]["topic_count"], 3);
}

#[test]
fn sheets_json_compact_format_limits_sheet_fields() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "sheets",
            "tests/fixtures/xmind/minimal.xmind",
            "--json",
            "--format",
            "compact-json",
            "--fields",
            "id,title,root_topic_id",
        ])
        .output()
        .expect("sheets command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json sheets output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "sheets");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["result"]["sheets"][0]["id"], "sheet-roadmap");
    assert_eq!(body["result"]["sheets"][0]["title"], "Roadmap");
    assert_eq!(body["result"]["sheets"][0]["root_topic_id"], "topic-root");
    assert!(body["result"]["sheets"][0].get("index").is_none());
    assert!(body["result"]["sheets"][0].get("topic_count").is_none());
}
