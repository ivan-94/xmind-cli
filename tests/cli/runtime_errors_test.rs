use assert_cmd::Command;
use serde_json::Value;

#[test]
fn json_validate_missing_workbook_returns_file_not_found_envelope() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["validate", "tests/fixtures/xmind/missing.xmind", "--json"])
        .output()
        .expect("validate command runs");

    assert_eq!(output.status.code(), Some(3));
    assert!(
        output.stderr.is_empty(),
        "json runtime errors should be emitted on stdout: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "validate");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/missing.xmind");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert_eq!(body["warnings"], Value::Array(vec![]));
    assert_eq!(body["error"]["code"], "file_not_found");
    assert_eq!(body["error"]["retryable"], true);
    assert_eq!(body["error"]["path"], "tests/fixtures/xmind/missing.xmind");
    assert_eq!(body["error"]["exit_code"], 3);
}

#[test]
fn json_validate_malformed_workbook_returns_parse_failed_envelope() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["validate", "tests/fixtures/xmind/malformed.xmind", "--json"])
        .output()
        .expect("validate command runs");

    assert_eq!(output.status.code(), Some(4));
    assert!(
        output.stderr.is_empty(),
        "json runtime errors should be emitted on stdout: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "validate");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/malformed.xmind");
    assert_eq!(body["error"]["code"], "parse_failed");
    assert_eq!(body["error"]["retryable"], false);
    assert_eq!(
        body["error"]["path"],
        "tests/fixtures/xmind/malformed.xmind"
    );
    assert_eq!(body["error"]["exit_code"], 4);
}
