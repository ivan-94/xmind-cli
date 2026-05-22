use assert_cmd::Command;
use serde_json::Value;
use std::fs;

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
