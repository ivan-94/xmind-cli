use assert_cmd::Command;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;

#[test]
fn human_runtime_errors_respect_no_color() {
    let colored_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["validate", "tests/fixtures/xmind/missing.xmind"])
        .output()
        .expect("validate command runs");

    assert_eq!(colored_output.status.code(), Some(3));
    let colored_stderr = String::from_utf8_lossy(&colored_output.stderr);
    assert!(
        colored_stderr.contains("\u{1b}[31mvalidate\u{1b}[0m"),
        "human runtime errors should color the command name by default: {colored_stderr}"
    );

    let plain_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "validate",
            "tests/fixtures/xmind/missing.xmind",
            "--no-color",
        ])
        .output()
        .expect("validate command runs");

    assert_eq!(plain_output.status.code(), Some(3));
    let plain_stderr = String::from_utf8_lossy(&plain_output.stderr);
    assert!(
        !plain_stderr.contains("\u{1b}["),
        "no-color human errors should not include ANSI escapes: {plain_stderr}"
    );
    assert!(
        plain_stderr.starts_with("validate: Workbook not found"),
        "no-color human errors should keep the plain command prefix: {plain_stderr}"
    );
}

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

#[test]
fn json_validate_unsupported_workbook_variant_returns_unsupported_format() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("legacy.xmind");
    write_unsupported_xmind_variant(&workbook);
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["validate", &workbook_arg, "--json"])
        .output()
        .expect("validate command runs");

    assert_eq!(output.status.code(), Some(11));
    assert!(
        output.stderr.is_empty(),
        "json runtime errors should be emitted on stdout: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "validate");
    assert_eq!(body["workbook"], workbook_arg);
    assert_eq!(body["error"]["code"], "unsupported_format");
    assert_eq!(body["error"]["retryable"], false);
    assert_eq!(body["error"]["exit_code"], 11);
    assert_eq!(body["error"]["path"], workbook_arg);
    assert!(body["error"]["suggested_fix"]
        .as_str()
        .expect("suggested fix is a string")
        .contains("Open and re-save"));
}

fn write_unsupported_xmind_variant(path: &Path) {
    let file = File::create(path).expect("unsupported workbook fixture is created");
    let mut writer = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    writer
        .start_file("content.xml", options)
        .expect("content.xml entry starts");
    writer
        .write_all(b"<xmap-content></xmap-content>")
        .expect("content.xml is written");
    writer
        .finish()
        .expect("unsupported workbook zip is finished");
}
