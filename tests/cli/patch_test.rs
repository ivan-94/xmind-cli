use assert_cmd::Command;
use serde_json::Value;

#[test]
fn patch_dry_run_add_tree_reports_structured_diff_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            "tests/fixtures/xmind/minimal.xmind",
            "--ops",
            "docs/examples/patch-add-tree.yaml",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("patch command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json patch output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "patch");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["will_change"], true);
    assert_eq!(body["result"]["summary"]["added"], 3);
    assert_eq!(body["result"]["summary"]["updated"], 0);
    assert_eq!(body["result"]["summary"]["deleted"], 0);
    assert_eq!(body["result"]["summary"]["moved"], 0);
    assert_eq!(body["result"]["operations"][0]["index"], 0);
    assert_eq!(body["result"]["operations"][0]["op"], "add_tree");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "added");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["diff"][1]["path"], "/Q2/支付能力/收银台");
    assert_eq!(body["result"]["diff"][2]["path"], "/Q2/支付能力/退款");

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
fn patch_json_dry_run_add_tree_reports_structured_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("ops.json");
    std::fs::write(
        &ops,
        r#"
{
  "ops": [
    {
      "op": "add_tree",
      "parent": "path:/Q2",
      "tree": {
        "title": "支付能力",
        "children": [
          { "title": "收银台" },
          { "title": "退款" }
        ]
      }
    }
  ]
}
"#,
    )
    .expect("patch fixture is written");
    let ops_arg = ops.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            "tests/fixtures/xmind/minimal.xmind",
            "--ops",
            &ops_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("patch command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "patch");
    assert_eq!(body["result"]["summary"]["added"], 3);
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["diff"][2]["path"], "/Q2/支付能力/退款");
}

#[test]
fn patch_invalid_operation_reports_operation_index() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            "tests/fixtures/xmind/minimal.xmind",
            "--ops",
            "tests/fixtures/patch/unsupported-op.yaml",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("patch command runs");

    assert_eq!(output.status.code(), Some(7));
    assert!(
        output.stderr.is_empty(),
        "json patch errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "patch");
    assert_eq!(body["error"]["code"], "invalid_patch");
    assert_eq!(body["error"]["operation_index"], 0);
    assert_eq!(body["error"]["operation"], "sort_children");
    assert_eq!(body["error"]["exit_code"], 7);
}

#[test]
fn patch_json_extension_rejects_yaml_syntax() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("ops.json");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: add_tree
    parent: path:/Q2
    tree:
      title: 支付能力
"#,
    )
    .expect("patch fixture is written");
    let ops_arg = ops.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            "tests/fixtures/xmind/minimal.xmind",
            "--ops",
            &ops_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("patch command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_patch");
    assert!(body["error"]["message"]
        .as_str()
        .is_some_and(|message| message.starts_with("Patch file JSON is invalid:")));
}
