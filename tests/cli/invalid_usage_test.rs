use assert_cmd::Command;
use serde_json::Value;

#[test]
fn json_invalid_usage_returns_agent_error_envelope_on_stdout() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", "roadmap.xmind", "--depth", "not-a-number", "--json"])
        .output()
        .expect("invalid usage command runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        output.stderr.is_empty(),
        "json invalid usage should not replace the JSON envelope with stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "tree");
    assert_eq!(body["workbook"], "roadmap.xmind");
    assert_eq!(body["error"]["code"], "invalid_usage");
    assert_eq!(body["error"]["retryable"], true);
    assert_eq!(body["error"]["exit_code"], 2);
    assert!(body["error"]["message"].as_str().is_some_and(|message| {
        message.contains("invalid value") || message.contains("invalid digit")
    }));
    assert!(body["error"]["suggested_fix"]
        .as_str()
        .is_some_and(|fix| fix.contains("Correct")));
}

#[test]
fn json_unknown_fields_return_field_path_diagnostic() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--title",
            "Payment",
            "--fields",
            "id,unknown",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        output.stderr.is_empty(),
        "json field validation should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "find");
    assert_eq!(body["error"]["code"], "invalid_usage");
    assert_eq!(body["error"]["field_path"], "fields");
    assert_eq!(body["error"]["details"]["field"], "unknown");
    assert_eq!(body["error"]["exit_code"], 2);
}
