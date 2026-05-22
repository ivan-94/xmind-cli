use assert_cmd::Command;
use serde_json::Value;

#[test]
fn validate_json_reports_readable_workbook_as_valid() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["validate", "tests/fixtures/xmind/minimal.xmind", "--json"])
        .output()
        .expect("validate command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json validate output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "validate");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert_eq!(body["warnings"], Value::Array(vec![]));
    assert_eq!(body["result"]["valid"], true);
    assert_eq!(body["result"]["warnings"], Value::Array(vec![]));
    assert_eq!(body["result"]["errors"], Value::Array(vec![]));
}

#[test]
fn validate_json_accepts_strict_for_readable_workbook() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "validate",
            "tests/fixtures/xmind/minimal.xmind",
            "--strict",
            "--json",
        ])
        .output()
        .expect("validate command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json validate output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "validate");
    assert_eq!(body["result"]["valid"], true);
    assert_eq!(body["result"]["warnings"], Value::Array(vec![]));
    assert_eq!(body["result"]["errors"], Value::Array(vec![]));
}
