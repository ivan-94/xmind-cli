use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;

#[test]
fn inspect_json_summarizes_workbook_without_printing_tree() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "inspect",
            "tests/fixtures/xmind/multiple-sheets.xmind",
            "--json",
        ])
        .output()
        .expect("inspect command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json inspect output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "inspect");
    assert_eq!(
        body["workbook"],
        "tests/fixtures/xmind/multiple-sheets.xmind"
    );
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["format"], "xmind-zen");
    assert_eq!(body["result"]["sheet_count"], 2);
    assert_eq!(body["result"]["sheets"][0]["id"], "sheet-roadmap");
    assert_eq!(body["result"]["sheets"][0]["title"], "Roadmap");
    assert_eq!(body["result"]["sheets"][0]["topic_count"], 2);
    assert_eq!(body["result"]["sheets"][1]["id"], "sheet-backlog");
    assert_eq!(body["result"]["sheets"][1]["title"], "Backlog");
    assert_eq!(body["result"]["sheets"][1]["topic_count"], 2);
    assert_eq!(body["result"]["capabilities"]["can_read_topics"], true);
    assert_eq!(
        body["result"]["capabilities"]["can_preserve_unknown"],
        false
    );
    assert!(body["result"].get("root").is_none());
}

#[test]
fn inspect_json_compact_format_limits_workbook_fields() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "inspect",
            "tests/fixtures/xmind/multiple-sheets.xmind",
            "--json",
            "--format",
            "compact-json",
            "--fields",
            "format,sheet_count",
        ])
        .output()
        .expect("inspect command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json inspect output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "inspect");
    assert_eq!(body["result"]["format"], "xmind-zen");
    assert_eq!(body["result"]["sheet_count"], 2);
    assert!(body["result"].get("sheets").is_none());
    assert!(body["result"].get("capabilities").is_none());
}

#[test]
fn inspect_json_reports_unknown_package_entry_preservation() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("with-extra-entry.xmind");
    write_xmind_with_extra_entry(&workbook);
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["inspect", &workbook_arg, "--json"])
        .output()
        .expect("inspect command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json inspect output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "inspect");
    assert_eq!(body["result"]["capabilities"]["can_preserve_unknown"], true);
}

#[test]
fn inspect_json_reports_unknown_json_field_preservation() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("with-unknown-json.xmind");
    write_xmind_with_unknown_json_field(&workbook);
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["inspect", &workbook_arg, "--json"])
        .output()
        .expect("inspect command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json inspect output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "inspect");
    assert_eq!(body["result"]["capabilities"]["can_preserve_unknown"], true);
}

#[test]
fn inspect_json_counts_resource_entries() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("with-resource.xmind");
    write_xmind_with_resource_entry(&workbook);
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["inspect", &workbook_arg, "--json"])
        .output()
        .expect("inspect command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json inspect output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "inspect");
    assert_eq!(body["result"]["resources_count"], 1);
}

#[test]
fn inspect_quiet_suppresses_human_success_output_without_suppressing_json() {
    let human_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "inspect",
            "tests/fixtures/xmind/multiple-sheets.xmind",
            "--quiet",
        ])
        .output()
        .expect("inspect command runs");

    assert_eq!(human_output.status.code(), Some(0));
    assert!(
        human_output.stdout.is_empty(),
        "quiet human output should suppress success text: {}",
        String::from_utf8_lossy(&human_output.stdout)
    );
    assert!(
        human_output.stderr.is_empty(),
        "quiet human output should not emit stderr on success: {}",
        String::from_utf8_lossy(&human_output.stderr)
    );

    let json_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "inspect",
            "tests/fixtures/xmind/multiple-sheets.xmind",
            "--quiet",
            "--json",
        ])
        .output()
        .expect("inspect command runs");

    assert_eq!(json_output.status.code(), Some(0));
    assert!(
        !json_output.stdout.is_empty(),
        "quiet must not suppress JSON stdout"
    );

    let body: Value = serde_json::from_slice(&json_output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "inspect");
}

fn write_xmind_with_extra_entry(path: &Path) {
    let content =
        fs::read_to_string("tests/fixtures/xmind/minimal-content.json").expect("fixture readable");
    let file = File::create(path).expect("workbook fixture is created");
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default();

    zip.start_file("content.json", options)
        .expect("content entry starts");
    zip.write_all(content.as_bytes())
        .expect("content entry is written");
    zip.start_file("metadata.json", options)
        .expect("metadata entry starts");
    zip.write_all(br#"{"vendor":true}"#)
        .expect("metadata entry is written");
    zip.finish().expect("workbook fixture is finalized");
}

fn write_xmind_with_unknown_json_field(path: &Path) {
    let content =
        fs::read_to_string("tests/fixtures/xmind/minimal-content.json").expect("fixture readable");
    let mut content: Value = serde_json::from_str(&content).expect("fixture is JSON");
    content[0]["rootTopic"]["unknownVendorField"] = serde_json::json!({ "preserve": true });
    write_xmind_content(
        path,
        &serde_json::to_string_pretty(&content).expect("content encodes"),
    );
}

fn write_xmind_with_resource_entry(path: &Path) {
    let content =
        fs::read_to_string("tests/fixtures/xmind/minimal-content.json").expect("fixture readable");
    let file = File::create(path).expect("workbook fixture is created");
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default();

    zip.start_file("content.json", options)
        .expect("content entry starts");
    zip.write_all(content.as_bytes())
        .expect("content entry is written");
    zip.start_file("resources/payment.png", options)
        .expect("resource entry starts");
    zip.write_all(b"png-bytes")
        .expect("resource entry is written");
    zip.finish().expect("workbook fixture is finalized");
}

fn write_xmind_content(path: &Path, content: &str) {
    let file = File::create(path).expect("workbook fixture is created");
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default();

    zip.start_file("content.json", options)
        .expect("content entry starts");
    zip.write_all(content.as_bytes())
        .expect("content entry is written");
    zip.finish().expect("workbook fixture is finalized");
}
