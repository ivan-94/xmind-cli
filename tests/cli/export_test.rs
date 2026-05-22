use assert_cmd::Command;
use serde_json::Value;

#[test]
fn export_format_json_writes_raw_tree_payload_to_stdout() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/minimal.xmind",
            "--format",
            "json",
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["sheet"], "Roadmap");
    assert_eq!(body["root"]["title"], "Roadmap");
    assert_eq!(body["root"]["children"][0]["title"], "Q2");
    assert!(body.get("ok").is_none(), "raw export is not an envelope");
}
