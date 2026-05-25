use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::FileOptions;

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
fn patch_dry_run_add_reports_single_topic_diff_without_writing() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("add.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: add
    parent: path:/Q2
    title: Refund
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
    assert_eq!(body["result"]["summary"]["added"], 1);
    assert_eq!(body["result"]["operations"][0]["index"], 0);
    assert_eq!(body["result"]["operations"][0]["op"], "add");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "added");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Refund");

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
        tree["result"]["root"]["children"][0]["children"]
            .as_array()
            .expect("children is an array")
            .len(),
        1
    );
}

#[test]
fn patch_dry_run_resolves_later_operations_against_working_copy() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("working-copy.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: add
    parent: path:/Q2
    title: Working Copy Topic
  - op: set
    node: path:/Q2/Working Copy Topic
    fields:
      note: Updated after creation
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
    assert_eq!(body["result"]["summary"]["added"], 1);
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "add");
    assert_eq!(body["result"]["operations"][1]["op"], "set");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Working Copy Topic");
    assert_eq!(body["result"]["diff"][1]["path"], "/Q2/Working Copy Topic");
}

#[test]
fn patch_apply_writes_workbook_and_preserves_unknown_package_entries() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("patch-apply-preserve.xmind");
    write_xmind_with_package_entry(&workbook, "metadata.json", br#"{"vendor":true}"#);
    let ops = temp_dir.path().join("apply.yaml");
    fs::write(
        &ops,
        r#"
ops:
  - op: add
    parent: path:/Q2
    title: Preserved Apply
"#,
    )
    .expect("patch fixture is written");
    let workbook_arg = workbook.to_string_lossy().into_owned();
    let ops_arg = ops.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            &workbook_arg,
            "--ops",
            &ops_arg,
            "--apply",
            "--json",
        ])
        .output()
        .expect("patch command runs");

    assert_eq!(
        output.status.code(),
        Some(0),
        "patch apply should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["operations"][0]["status"], "applied");
    assert_eq!(body["result"]["summary"]["added"], 1);

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "3"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    let titles = serde_json::to_string(&tree["result"]["root"]).expect("tree serializes");
    assert!(titles.contains("Preserved Apply"));
    assert_eq!(
        read_zip_entry(&workbook, "metadata.json"),
        br#"{"vendor":true}"#
    );
}

#[test]
fn patch_dry_run_set_reports_updated_topic_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("set.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: set
    node: path:/Q2
    fields:
      title: Q2 Roadmap
      note: Updated scope
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
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["operations"][0]["index"], 0);
    assert_eq!(body["result"]["operations"][0]["op"], "set");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2 Roadmap");
}

#[test]
fn patch_dry_run_set_same_value_is_idempotent() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("set-same-title.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: set
    node: path:/Q2/Payment
    fields:
      title: Payment
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
    assert_eq!(body["result"]["will_change"], false);
    assert_eq!(body["result"]["summary"]["updated"], 0);
    assert_eq!(body["result"]["operations"][0]["op"], "set");
    assert_eq!(
        body["result"]["diff"]
            .as_array()
            .expect("diff is array")
            .len(),
        0
    );
}

#[test]
fn patch_dry_run_replace_tree_reports_deleted_and_added_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("replace-tree.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: replace_tree
    node: path:/Q2/Payment
    tree:
      title: Billing
      children:
        - title: Checkout
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
    assert_eq!(body["result"]["summary"]["added"], 2);
    assert_eq!(body["result"]["summary"]["deleted"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "replace_tree");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "deleted");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["diff"][1]["event"], "added");
    assert_eq!(body["result"]["diff"][1]["path"], "/Q2/Billing");
    assert_eq!(body["result"]["diff"][2]["path"], "/Q2/Billing/Checkout");

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
fn patch_dry_run_merge_tree_updates_matched_topics_and_adds_missing_topics() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("merge-tree.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: merge_tree
    target: path:/Q2/Payment
    match_by: title_path
    tree:
      title: Payment
      note: Updated payment scope
      children:
        - title: Checkout
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
    assert_eq!(body["result"]["summary"]["added"], 1);
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["summary"]["deleted"], 0);
    assert_eq!(body["result"]["operations"][0]["op"], "merge_tree");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["diff"][1]["event"], "added");
    assert_eq!(body["result"]["diff"][1]["path"], "/Q2/Payment/Checkout");

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
fn patch_dry_run_merge_tree_prune_reports_unmatched_descendants_deleted() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("merge-tree-prune.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: merge_tree
    target: path:/Q2
    prune: true
    tree:
      title: Q2
      children: []
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
    assert_eq!(body["result"]["summary"]["deleted"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "merge_tree");
    assert_eq!(body["result"]["diff"][0]["event"], "deleted");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");
}

#[test]
fn patch_conflict_reports_later_operation_targeting_pruned_topic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("prune-conflict.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: merge_tree
    target: path:/Q2
    prune: true
    tree:
      title: Q2
      children: []
  - op: set
    node: path:/Q2/Payment
    fields:
      note: Should conflict because Payment was pruned
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

    assert_eq!(output.status.code(), Some(8));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "patch_conflict");
    assert_eq!(body["error"]["operation_index"], 1);
    assert_eq!(body["error"]["operation"], "set");
    assert_eq!(body["error"]["selector"], "path:/Q2/Payment");
    assert_eq!(body["error"]["field_path"], "ops[1].node");
}

#[test]
fn patch_dry_run_merge_tree_match_by_id_updates_existing_topic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("merge-tree-id.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: merge_tree
    target: path:/Q2
    match_by: id
    tree:
      id: topic-q2
      title: Q2
      children:
        - id: topic-payment
          title: Payment
          note: Updated by id
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
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "merge_tree");
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");
}

#[test]
fn patch_dry_run_merge_tree_match_by_path_updates_existing_topic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("merge-tree-path.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: merge_tree
    target: path:/Q2
    match_by: path
    tree:
      path: /Q2
      title: Q2
      children:
        - path: /Q2/Payment
          title: Payment
          note: Updated by path
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
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "merge_tree");
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");
}

#[test]
fn patch_dry_run_merge_tree_match_by_title_updates_existing_topic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("merge-tree-title.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: merge_tree
    target: path:/Q2
    match_by: title
    tree:
      title: Q2
      children:
        - title: Payment
          note: Updated by title
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
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "merge_tree");
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");
}

#[test]
fn patch_dry_run_delete_reports_deleted_subtree_diff_without_writing() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("delete.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: delete
    node: path:/Q2
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
    assert_eq!(body["result"]["summary"]["deleted"], 2);
    assert_eq!(body["result"]["operations"][0]["op"], "delete");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "deleted");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2");
    assert_eq!(body["result"]["diff"][1]["event"], "deleted");
    assert_eq!(body["result"]["diff"][1]["path"], "/Q2/Payment");

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
fn patch_dry_run_delete_children_only_reports_descendant_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("delete-children-only.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: delete
    node: path:/Q2
    children_only: true
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
    assert_eq!(body["result"]["summary"]["deleted"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "delete");
    assert_eq!(body["result"]["diff"][0]["event"], "deleted");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");
}

#[test]
fn patch_dry_run_delete_promote_children_reports_deleted_and_moved_diff() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("delete-promote-children.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: delete
    node: path:/Q2
    promote_children: true
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
    assert_eq!(body["result"]["summary"]["deleted"], 1);
    assert_eq!(body["result"]["summary"]["moved"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "delete");
    assert_eq!(body["result"]["diff"][0]["event"], "deleted");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2");
    assert_eq!(body["result"]["diff"][1]["event"], "moved");
    assert_eq!(body["result"]["diff"][1]["from"], "/Q2/Payment");
    assert_eq!(body["result"]["diff"][1]["to"], "/Payment");
}

#[test]
fn patch_dry_run_move_reports_moved_diff_without_writing() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("move.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: move
    node: id:topic-payment-q1
    to: root
"#,
    )
    .expect("patch fixture is written");
    let ops_arg = ops.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            "tests/fixtures/xmind/duplicate-titles.xmind",
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
    assert_eq!(body["result"]["summary"]["moved"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "move");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "moved");
    assert_eq!(body["result"]["diff"][0]["from"], "/Q1/Payment");
    assert_eq!(body["result"]["diff"][0]["to"], "/Payment");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/duplicate-titles.xmind",
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
fn patch_dry_run_copy_reports_added_subtree_diff_without_writing() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("copy.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: copy
    node: id:topic-q1
    to: id:topic-q2
    title: Q1 Copy
"#,
    )
    .expect("patch fixture is written");
    let ops_arg = ops.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            "tests/fixtures/xmind/duplicate-titles.xmind",
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
    assert_eq!(body["result"]["summary"]["added"], 2);
    assert_eq!(body["result"]["operations"][0]["op"], "copy");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "added");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Q1 Copy");
    assert_eq!(body["result"]["diff"][1]["event"], "added");
    assert_eq!(body["result"]["diff"][1]["path"], "/Q2/Q1 Copy/Payment");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--json",
            "--depth",
            "2",
        ])
        .output()
        .expect("tree command runs after dry run");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(
        tree["result"]["root"]["children"][1]["children"]
            .as_array()
            .expect("Q2 children is an array")
            .len(),
        1
    );
}

#[test]
fn patch_copy_preserve_ids_reports_duplicate_id_conflict() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("copy-preserve-ids.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: copy
    node: id:topic-q1
    to: id:topic-q2
    preserve_ids: true
"#,
    )
    .expect("patch fixture is written");
    let ops_arg = ops.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--ops",
            &ops_arg,
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("patch command runs");

    assert_eq!(output.status.code(), Some(8));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "patch_conflict");
    assert_eq!(body["error"]["operation_index"], 0);
    assert_eq!(body["error"]["operation"], "copy");
    assert_eq!(body["error"]["field_path"], "ops[0].preserve_ids");
}

#[test]
fn patch_dry_run_ensure_path_reports_missing_segments_without_writing() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("ensure-path.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: ensure_path
    path: /Q2/Payment/Refunds
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
    assert_eq!(body["result"]["summary"]["added"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "ensure_path");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "added");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment/Refunds");

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
fn patch_dry_run_sort_children_reports_updated_parent_when_order_changes() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("sort-children.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: sort_children
    node: root
    by: title
    order: desc
"#,
    )
    .expect("patch fixture is written");
    let ops_arg = ops.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "patch",
            "tests/fixtures/xmind/duplicate-titles.xmind",
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
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["operations"][0]["op"], "sort_children");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--json",
            "--depth",
            "1",
        ])
        .output()
        .expect("tree command runs after dry run");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(tree["result"]["root"]["children"][0]["title"], "Q1");
}

#[test]
fn patch_dry_run_set_tree_metadata_adds_labels_recursively() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("set-tree-metadata.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: set_tree_metadata
    node: path:/Q2
    recursive: true
    add_labels:
      - MVP
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
    assert_eq!(body["result"]["summary"]["updated"], 2);
    assert_eq!(body["result"]["operations"][0]["op"], "set_tree_metadata");
    assert_eq!(body["result"]["operations"][0]["status"], "planned");
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2");
    assert_eq!(body["result"]["diff"][1]["event"], "updated");
    assert_eq!(body["result"]["diff"][1]["path"], "/Q2/Payment");

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
    assert_eq!(tree["result"]["root"]["children"][0]["title"], "Q2");
}

#[test]
fn patch_replace_tree_rejects_root_target() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("replace-root.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: replace_tree
    node: root
    tree:
      title: Replacement
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
    assert_eq!(body["error"]["operation_index"], 0);
    assert_eq!(body["error"]["operation"], "replace_tree");
    assert_eq!(
        body["error"]["message"],
        "replace_tree cannot target the root topic."
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
    assert_eq!(body["error"]["operation"], "unsupported_sort");
    assert_eq!(body["error"]["exit_code"], 7);
}

#[test]
fn patch_operation_diagnostic_reports_indexed_field_path() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("indexed-diagnostic.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: assert_exists
    node: path:/Q2
  - op: delete
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
    assert_eq!(body["error"]["operation_index"], 1);
    assert_eq!(body["error"]["operation"], "delete");
    assert_eq!(body["error"]["field_path"], "ops[1].node");
}

#[test]
fn patch_legacy_aliases_are_normalized_before_operation_diagnostics() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let cases = [
        ("delete_tree", "delete", "delete operation is missing node."),
        ("move_tree", "move", "move operation is missing node."),
        ("clone_tree", "copy", "copy operation is missing node."),
    ];

    for (alias, canonical, expected_message) in cases {
        let ops = temp_dir.path().join(format!("{alias}.yaml"));
        std::fs::write(
            &ops,
            format!(
                r#"
ops:
  - op: {alias}
"#
            ),
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
        assert_eq!(body["error"]["operation_index"], 0);
        assert_eq!(body["error"]["operation"], canonical);
        assert_eq!(body["error"]["message"], expected_message);
    }
}

#[test]
fn patch_assert_operations_pass_without_diff_when_expectations_hold() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("assertions.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: assert_exists
    node: path:/Q2
  - op: assert_not_exists
    node: path:/Q2/Deprecated
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
    assert_eq!(body["result"]["will_change"], false);
    assert_eq!(body["result"]["summary"]["added"], 0);
    assert_eq!(body["result"]["operations"][0]["op"], "assert_exists");
    assert_eq!(body["result"]["operations"][0]["status"], "passed");
    assert_eq!(body["result"]["operations"][1]["op"], "assert_not_exists");
    assert_eq!(body["result"]["operations"][1]["status"], "passed");
    assert_eq!(
        body["result"]["diff"]
            .as_array()
            .expect("diff is array")
            .len(),
        0
    );
}

#[test]
fn patch_assert_exists_reports_operation_index_when_missing() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("assert-exists-missing.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: assert_exists
    node: path:/Q2/Missing
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

    assert_eq!(output.status.code(), Some(5));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "not_found");
    assert_eq!(body["error"]["operation_index"], 0);
    assert_eq!(body["error"]["operation"], "assert_exists");
    assert_eq!(body["error"]["selector"], "path:/Q2/Missing");
}

#[test]
fn patch_assert_not_exists_reports_conflict_when_topic_exists() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let ops = temp_dir.path().join("assert-not-exists-conflict.yaml");
    std::fs::write(
        &ops,
        r#"
ops:
  - op: assert_not_exists
    node: path:/Q2
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

    assert_eq!(output.status.code(), Some(8));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "patch_conflict");
    assert_eq!(body["error"]["operation_index"], 0);
    assert_eq!(body["error"]["operation"], "assert_not_exists");
    assert_eq!(body["error"]["selector"], "path:/Q2");
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

fn write_xmind_with_package_entry(path: &Path, entry_name: &str, entry_bytes: &[u8]) {
    let file = fs::File::create(path).expect("workbook fixture is created");
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default();
    zip.start_file("content.json", options)
        .expect("content.json is created");
    zip.write_all(
        br#"[
  {
    "id": "sheet-roadmap",
    "title": "Roadmap",
    "rootTopic": {
      "id": "topic-root",
      "title": "Roadmap",
      "children": {
        "attached": [
          {
            "id": "topic-q2",
            "title": "Q2"
          }
        ]
      }
    }
  }
]"#,
    )
    .expect("content.json is written");
    zip.start_file(entry_name, options)
        .expect("package entry is created");
    zip.write_all(entry_bytes)
        .expect("package entry is written");
    zip.finish().expect("workbook fixture is finalized");
}

fn read_zip_entry(path: &Path, entry_name: &str) -> Vec<u8> {
    let file = fs::File::open(path).expect("workbook is readable");
    let mut zip = zip::ZipArchive::new(file).expect("workbook zip opens");
    let mut entry = zip.by_name(entry_name).expect("package entry exists");
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).expect("entry is readable");
    bytes
}
