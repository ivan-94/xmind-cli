use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use zip::write::FileOptions;

#[test]
fn add_tree_yaml_input_dry_run_reports_tree_diff_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json add-tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "add-tree");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["will_change"], true);
    assert_eq!(body["result"]["parent"]["path"], "/Q2");
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["summary"]["added"], 9);
    assert_eq!(body["result"]["diff"][0]["event"], "added");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/支付能力");
    assert_eq!(
        body["result"]["diff"][3]["path"],
        "/Q2/支付能力/收银台/优惠券抵扣"
    );
    assert_eq!(body["result"]["diff"][8]["path"], "/Q2/支付能力/对账");

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

#[test]
fn add_tree_json_input_dry_run_reports_tree_diff_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.json",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json add-tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "add-tree");
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["summary"]["added"], 8);
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/支付能力");
    assert_eq!(
        body["result"]["diff"][3]["path"],
        "/Q2/支付能力/收银台/优惠券抵扣"
    );
    assert_eq!(
        body["result"]["diff"][7]["path"],
        "/Q2/支付能力/退款/部分退款"
    );

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

#[test]
fn add_tree_human_output_distinguishes_dry_run_from_apply() {
    let dry_run = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--dry-run",
        ])
        .output()
        .expect("add-tree dry-run command runs");

    assert_eq!(dry_run.status.code(), Some(0));
    let dry_run_stdout = String::from_utf8_lossy(&dry_run.stdout);
    assert!(
        dry_run_stdout.contains("planned 9 added topics"),
        "dry-run human output should describe the planned change: {dry_run_stdout}"
    );

    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = copy_minimal_workbook(temp_dir.path(), "human-apply.xmind");
    let workbook_arg = workbook.to_string_lossy().into_owned();
    let apply = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            &workbook_arg,
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--apply",
        ])
        .output()
        .expect("add-tree apply command runs");

    assert_eq!(apply.status.code(), Some(0));
    let apply_stdout = String::from_utf8_lossy(&apply.stdout);
    assert!(
        apply_stdout.contains("applied 9 added topics"),
        "apply human output should describe the applied change: {apply_stdout}"
    );
    assert!(
        !apply_stdout.contains("planned"),
        "apply human output must not describe a committed write as planned: {apply_stdout}"
    );
}

#[test]
fn add_tree_yaml_input_apply_writes_backup_and_preserves_unknown_package_entries() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("with-unknown-entry.xmind");
    write_workbook_with_unknown_entry(&workbook);
    let workbook_arg = workbook.to_string_lossy().into_owned();
    let before_bytes = fs::read(&workbook).expect("workbook bytes are readable before apply");

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            &workbook_arg,
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--apply",
            "--backup",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "json add-tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "add-tree");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["summary"]["added"], 9);

    let backup_path = body["result"]["backup_path"]
        .as_str()
        .expect("backup path is reported when --backup is used");
    assert_eq!(
        fs::read(backup_path).expect("backup bytes are readable"),
        before_bytes
    );
    assert_eq!(
        read_zip_entry(&workbook, "vendor/unknown.json"),
        br#"{"vendor":true}"#
    );

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "3"])
        .output()
        .expect("tree command runs after add-tree apply");
    assert_eq!(tree_output.status.code(), Some(0));
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(
        tree["result"]["root"]["children"][0]["children"][1]["title"],
        "支付能力"
    );
}

#[test]
fn add_tree_json_input_apply_writes_subtree() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = copy_minimal_workbook(temp_dir.path(), "json-apply.xmind");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            &workbook_arg,
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.json",
            "--apply",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["result"]["summary"]["added"], 8);
    assert_tree_contains_title(&workbook_arg, "支付能力");
}

#[test]
fn add_tree_markdown_input_apply_writes_subtree() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = copy_minimal_workbook(temp_dir.path(), "markdown-apply.xmind");
    let markdown = temp_dir.path().join("outline.md");
    fs::write(
        &markdown,
        r#"# Payment Capability

## Checkout

## Refund
"#,
    )
    .expect("markdown input is written");
    let workbook_arg = workbook.to_string_lossy().into_owned();
    let markdown_arg = markdown.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            &workbook_arg,
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &markdown_arg,
            "--apply",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["result"]["summary"]["added"], 3);
    assert_tree_contains_title(&workbook_arg, "Payment Capability");
}

#[test]
fn add_tree_rejects_empty_nested_title() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("invalid-tree.yaml");
    fs::write(
        &input,
        r#"
title: Roadmap
children:
  - title: ""
"#,
    )
    .expect("invalid tree fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(7));
    assert!(
        output.stderr.is_empty(),
        "json add-tree errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "add-tree");
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(body["error"]["field_path"], "children[0].title");
}

fn copy_minimal_workbook(dir: &std::path::Path, file_name: &str) -> std::path::PathBuf {
    let workbook = dir.join(file_name);
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook)
        .expect("minimal workbook fixture is copied");
    workbook
}

fn write_workbook_with_unknown_entry(path: &std::path::Path) {
    let content_json = read_zip_entry(
        std::path::Path::new("tests/fixtures/xmind/minimal.xmind"),
        "content.json",
    );
    let file = fs::File::create(path).expect("workbook fixture is created");
    let mut writer = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    writer
        .start_file("content.json", options)
        .expect("content.json entry starts");
    writer
        .write_all(&content_json)
        .expect("content.json entry is written");
    writer
        .start_file("vendor/unknown.json", options)
        .expect("unknown entry starts");
    writer
        .write_all(br#"{"vendor":true}"#)
        .expect("unknown entry is written");
    writer.finish().expect("workbook fixture is finalized");
}

fn read_zip_entry(path: &std::path::Path, entry_name: &str) -> Vec<u8> {
    let file = fs::File::open(path).expect("zip file is opened");
    let mut archive = zip::ZipArchive::new(file).expect("zip file is readable");
    let mut entry = archive.by_name(entry_name).expect("zip entry exists");
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).expect("zip entry is read");
    bytes
}

fn assert_tree_contains_title(workbook_arg: &str, expected_title: &str) {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", workbook_arg, "--json", "--depth", "3"])
        .output()
        .expect("tree command runs after add-tree apply");
    assert_eq!(output.status.code(), Some(0));
    let tree: Value = serde_json::from_slice(&output.stdout).expect("tree stdout is JSON");
    let titles = serde_json::to_string(&tree["result"]["root"]).expect("tree serializes");
    assert!(
        titles.contains(expected_title),
        "tree output should contain {expected_title}: {titles}"
    );
}

#[test]
fn add_tree_preserves_optional_input_id_in_dry_run() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("id-tree.yaml");
    fs::write(
        &input,
        r#"
id: topic-payment-capability
title: Payment Capability
children:
  - id: topic-checkout
    title: Checkout
"#,
    )
    .expect("id tree fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["created_root"]["id"],
        "topic-payment-capability"
    );
    assert_eq!(
        body["result"]["created_root"]["path"],
        "/Q2/Payment Capability"
    );
    assert_eq!(body["result"]["summary"]["added"], 2);
}

#[test]
fn add_tree_preserves_image_field_in_dry_run() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("image-tree.yaml");
    fs::write(
        &input,
        r#"
title: Architecture
image:
  asset_id: xap:resources/architecture.png
  alt: Architecture diagram
  title: Architecture
"#,
    )
    .expect("image tree fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["created_root"]["image"]["asset_id"],
        "xap:resources/architecture.png"
    );
    assert_eq!(
        body["result"]["created_root"]["image"]["alt"],
        "Architecture diagram"
    );
    assert_eq!(
        body["result"]["created_root"]["image"]["title"],
        "Architecture"
    );
}

#[test]
fn add_tree_markdown_frontmatter_dry_run_uses_root_defaults() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("frontmatter.md");
    fs::write(
        &input,
        r#"---
title: Payment Capability
labels: [MVP]
markers: [priority-1]
---
"#,
    )
    .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["created_root"]["path"],
        "/Q2/Payment Capability"
    );
    assert_eq!(body["result"]["created_root"]["labels"][0], "MVP");
    assert_eq!(body["result"]["created_root"]["markers"][0], "priority-1");
    assert_eq!(body["result"]["summary"]["added"], 1);
}

#[test]
fn add_tree_markdown_heading_outline_dry_run_reports_tree_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("heading-outline.md");
    fs::write(
        &input,
        r#"---
labels: [MVP]
---

# Payment Capability

## Checkout

### Card Payment

## Refund
"#,
    )
    .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["created_root"]["path"],
        "/Q2/Payment Capability"
    );
    assert_eq!(body["result"]["created_root"]["labels"][0], "MVP");
    assert_eq!(body["result"]["summary"]["added"], 4);
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment Capability");
    assert_eq!(
        body["result"]["diff"][2]["path"],
        "/Q2/Payment Capability/Checkout/Card Payment"
    );
    assert_eq!(
        body["result"]["diff"][3]["path"],
        "/Q2/Payment Capability/Refund"
    );
}

#[test]
fn add_tree_markdown_list_outline_dry_run_reports_tree_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("list-outline.md");
    fs::write(
        &input,
        r#"- Payment Capability
  - Checkout
    - Card Payment
  - Refund
"#,
    )
    .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["created_root"]["path"],
        "/Q2/Payment Capability"
    );
    assert_eq!(body["result"]["summary"]["added"], 4);
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment Capability");
    assert_eq!(
        body["result"]["diff"][2]["path"],
        "/Q2/Payment Capability/Checkout/Card Payment"
    );
    assert_eq!(
        body["result"]["diff"][3]["path"],
        "/Q2/Payment Capability/Refund"
    );
}

#[test]
fn add_tree_markdown_ordered_list_outline_dry_run_reports_tree_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("ordered-outline.md");
    fs::write(
        &input,
        r#"1. Payment Capability
   1. Checkout
      1. Card Payment
   2. Refund
"#,
    )
    .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["created_root"]["path"],
        "/Q2/Payment Capability"
    );
    assert_eq!(body["result"]["summary"]["added"], 4);
    assert_eq!(
        body["result"]["diff"][2]["path"],
        "/Q2/Payment Capability/Checkout/Card Payment"
    );
    assert_eq!(
        body["result"]["diff"][3]["path"],
        "/Q2/Payment Capability/Refund"
    );
}

#[test]
fn add_tree_markdown_task_list_outline_dry_run_maps_task_markers() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("task-outline.md");
    fs::write(
        &input,
        r#"- [ ] Payment Capability
  - [x] Checkout
"#,
    )
    .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["created_root"]["path"],
        "/Q2/Payment Capability"
    );
    assert_eq!(body["result"]["created_root"]["markers"][0], "task-open");
    assert_eq!(
        body["result"]["diff"][1]["path"],
        "/Q2/Payment Capability/Checkout"
    );
    assert_eq!(body["result"]["summary"]["added"], 2);
}

#[test]
fn add_tree_markdown_heading_list_hybrid_dry_run_reports_tree_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("hybrid-outline.md");
    fs::write(
        &input,
        r#"# Roadmap

## Q2

- Payment Capability
  - Checkout
- Member Capability
"#,
    )
    .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/Roadmap");
    assert_eq!(body["result"]["summary"]["added"], 5);
    assert_eq!(
        body["result"]["diff"][2]["path"],
        "/Q2/Roadmap/Q2/Payment Capability"
    );
    assert_eq!(
        body["result"]["diff"][3]["path"],
        "/Q2/Roadmap/Q2/Payment Capability/Checkout"
    );
    assert_eq!(
        body["result"]["diff"][4]["path"],
        "/Q2/Roadmap/Q2/Member Capability"
    );
}

#[test]
fn add_tree_markdown_heading_paragraph_maps_to_note() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("heading-notes.md");
    fs::write(
        &input,
        r#"# Payment Capability

Q2 core delivery scope.

## Checkout

Checkout scope.
"#,
    )
    .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["created_root"]["note"],
        "Q2 core delivery scope."
    );
    assert_eq!(body["result"]["summary"]["added"], 2);
}

#[test]
fn add_tree_markdown_rejects_inline_metadata() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("inline-metadata.md");
    fs::write(&input, "- Payment Capability {labels: [MVP]}\n")
        .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(
        body["error"]["message"],
        "Inline metadata is not supported in Markdown input."
    );
}

#[test]
fn add_tree_markdown_heading_mode_rejects_list_outline() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("list-outline.md");
    fs::write(&input, "- Payment Capability\n  - Checkout\n").expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--markdown-mode",
            "heading",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(
        body["error"]["message"],
        "Markdown heading mode does not accept list items."
    );
}

#[test]
fn add_tree_markdown_list_mode_rejects_heading_outline() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("heading-outline.md");
    fs::write(&input, "# Payment Capability\n\n## Checkout\n")
        .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--markdown-mode",
            "list",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(
        body["error"]["message"],
        "Markdown list mode does not accept headings."
    );
}

#[test]
fn add_tree_markdown_rejects_unknown_mode() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("heading-outline.md");
    fs::write(&input, "# Payment Capability\n").expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--markdown-mode",
            "unknown",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        output.stderr.is_empty(),
        "json invalid usage should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_usage");
    assert!(body["error"]["message"]
        .as_str()
        .is_some_and(|message| message.contains("invalid value")));
}

#[test]
fn add_tree_markdown_accepts_all_documented_modes() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let cases = [
        ("heading", "# Payment Capability\n\n## Checkout\n"),
        ("list", "- Payment Capability\n  - Checkout\n"),
        ("hybrid", "# Payment Capability\n\n- Checkout\n"),
        ("auto", "# Payment Capability\n\n- Checkout\n"),
    ];

    for (mode, markdown) in cases {
        let input = temp_dir.path().join(format!("{mode}.md"));
        fs::write(&input, markdown).expect("markdown fixture is written");
        let input_arg = input.to_string_lossy().into_owned();

        let output = Command::cargo_bin("xmind")
            .expect("xmind binary is built for CLI tests")
            .args([
                "add-tree",
                "tests/fixtures/xmind/minimal.xmind",
                "--parent",
                "path:/Q2",
                "--from-markdown",
                &input_arg,
                "--markdown-mode",
                mode,
                "--dry-run",
                "--json",
            ])
            .output()
            .expect("add-tree command runs");

        assert_eq!(
            output.status.code(),
            Some(0),
            "{mode} mode should accept its documented outline form: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
        assert_eq!(body["ok"], true);
        assert_eq!(
            body["result"]["created_root"]["path"],
            "/Q2/Payment Capability"
        );
    }
}

#[test]
fn add_tree_markdown_rejects_skipped_heading_level_with_line_diagnostic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("skipped-heading.md");
    fs::write(&input, "# Payment Capability\n\n### Checkout\n")
        .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(
        body["error"]["message"],
        "Markdown heading levels cannot skip from 1 to 3 at line 3."
    );
}

#[test]
fn add_tree_markdown_rejects_inconsistent_list_indent_with_line_diagnostic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("bad-list-indent.md");
    fs::write(&input, "- Payment Capability\n   - Checkout\n")
        .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(
        body["error"]["message"],
        "Markdown unordered list indentation must use multiples of 2 spaces at line 2."
    );
}

#[test]
fn add_tree_markdown_rejects_empty_list_title_with_line_diagnostic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("empty-list-title.md");
    fs::write(&input, "- [x] \n").expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(
        body["error"]["message"],
        "Markdown list item title is empty at line 1."
    );
}

#[test]
fn add_tree_markdown_rejects_multiple_roots_with_line_diagnostic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let input = temp_dir.path().join("multiple-roots.md");
    fs::write(&input, "- Payment Capability\n- Member Capability\n")
        .expect("markdown fixture is written");
    let input_arg = input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--from-markdown",
            &input_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(
        body["error"]["message"],
        "Markdown outline must contain one top-level root; second root starts at line 2."
    );
}
