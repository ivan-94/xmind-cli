use assert_cmd::Command;
use serde_json::Value;
use std::fs;

#[test]
fn export_output_writes_payload_to_file_without_stdout() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let output_path = temp_dir.path().join("roadmap.md");
    let output_arg = output_path.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/minimal.xmind",
            "--format",
            "markdown",
            "--output",
            &output_arg,
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stdout.is_empty(),
        "export --output should not emit raw payload to stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(output_path).expect("output file is written"),
        "# Roadmap\n\n## Q2\n\n### Payment\n"
    );
}

#[test]
fn export_overwrite_replaces_existing_output_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let output_path = temp_dir.path().join("roadmap.md");
    std::fs::write(&output_path, "old content\n").expect("existing output is written");
    let output_arg = output_path.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/minimal.xmind",
            "--format",
            "markdown",
            "--output",
            &output_arg,
            "--overwrite",
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(output_path).expect("output file is overwritten"),
        "# Roadmap\n\n## Q2\n\n### Payment\n"
    );
}

#[test]
fn export_output_without_overwrite_rejects_existing_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let output_path = temp_dir.path().join("roadmap.md");
    std::fs::write(&output_path, "old content\n").expect("existing output is written");
    let output_arg = output_path.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/minimal.xmind",
            "--format",
            "markdown",
            "--output",
            &output_arg,
            "--json",
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(10));
    assert!(
        output.stderr.is_empty(),
        "json export errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(output_path).expect("existing output remains readable"),
        "old content\n"
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "write_failed");
}

#[test]
fn export_json_wraps_payload_in_success_envelope() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/minimal.xmind",
            "--format",
            "markdown",
            "--json",
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
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "export");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["format"], "markdown");
    assert_eq!(
        body["result"]["content"],
        "# Roadmap\n\n## Q2\n\n### Payment\n"
    );
    assert!(body.get("error").is_none());
}

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

#[test]
fn export_format_markdown_writes_heading_outline_to_stdout() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/minimal.xmind",
            "--format",
            "markdown",
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        "# Roadmap\n\n## Q2\n\n### Payment\n"
    );
}

#[test]
fn export_format_markdown_preserves_topic_hyperlinks() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/metadata.xmind",
            "--format",
            "markdown",
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let markdown = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(markdown.contains("[Payment](<https://example.com/payments>)"));
}

#[test]
fn export_format_markdown_writes_topic_notes_as_body_text() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/metadata.xmind",
            "--format",
            "markdown",
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let markdown = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(markdown.contains(
        "### [Payment](<https://example.com/payments>)\n\nSupports card payments and refund workflows."
    ));
}

#[test]
fn export_format_markdown_writes_parent_notes_before_child_topics() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("parent-note.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let set_note_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "path:/Q2",
            "--note",
            "Q2 first paragraph.\n\nQ2 second paragraph.",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set note command runs");
    assert_eq!(set_note_output.status.code(), Some(0));
    assert!(
        set_note_output.stderr.is_empty(),
        "set note should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&set_note_output.stderr)
    );

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["export", &workbook_arg, "--format", "markdown"])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        "# Roadmap\n\n## Q2\n\nQ2 first paragraph.\n\nQ2 second paragraph.\n\n### Payment\n"
    );
}

#[test]
fn export_format_markdown_escapes_topic_hyperlink_syntax() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("markdown-link-escaping.xmind");
    fs::copy("tests/fixtures/xmind/metadata.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let title_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--title",
            r"Pay [Beta]\Core]",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set title command runs");
    assert_eq!(title_output.status.code(), Some(0));
    assert!(
        title_output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&title_output.stderr)
    );

    let hyperlink_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--hyperlink",
            "https://example.com/payments?a>b",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set hyperlink command runs");
    assert_eq!(hyperlink_output.status.code(), Some(0));
    assert!(
        hyperlink_output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&hyperlink_output.stderr)
    );

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["export", &workbook_arg, "--format", "markdown"])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let markdown = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(markdown.contains(r"### [Pay \[Beta\]\\Core\]](<https://example.com/payments?a\>b>)"));
}

#[test]
fn export_format_outline_writes_indented_outline_to_stdout() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/minimal.xmind",
            "--format",
            "outline",
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        "Roadmap\n  Q2\n    Payment\n"
    );
}

#[test]
fn export_format_text_writes_default_readable_outline_to_stdout() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/minimal.xmind",
            "--format",
            "text",
        ])
        .output()
        .expect("export command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        "Roadmap\n  Q2\n    Payment\n"
    );
}

#[test]
fn export_format_assets_lists_resource_ids_to_stdout() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/topic-image.xmind",
            "--format",
            "assets",
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
    assert_eq!(body["format"], "assets");
    assert_eq!(body["assets"][0]["asset_id"], "xap:resources/payment.png");
}

#[test]
fn export_format_assets_output_writes_embedded_resources_to_directory() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let output_dir = temp_dir.path().join("assets");
    let output_arg = output_dir.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            "tests/fixtures/xmind/topic-image.xmind",
            "--format",
            "assets",
            "--output",
            &output_arg,
            "--json",
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
    assert_eq!(body["ok"], true);
    assert_eq!(body["result"]["output"], output_arg);

    let exported_image = output_dir.join("resources/payment.png");
    assert_eq!(
        std::fs::read(exported_image).expect("embedded resource is exported"),
        b"png-bytes"
    );
}
