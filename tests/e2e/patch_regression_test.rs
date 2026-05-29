#[path = "support.rs"]
#[allow(dead_code)]
mod support;

use support::{
    assert_success_envelope, copy_fixture, run_json, run_json_error, temp_file, validate_workbook,
    MINIMAL_FIXTURE,
};

fn topic_child_titles(body: &serde_json::Value) -> Vec<String> {
    body["result"]["topic"]["children"]
        .as_array()
        .expect("topic children are returned")
        .iter()
        .filter_map(|child| child["title"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn patch_merge_tree_apply_updates_id_selected_target() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for patch input");
    let ops = temp_file(
        temp_dir.path(),
        "merge-id-target.yaml",
        r#"ops:
  - op: merge_tree
    target: id:topic-payment
    match_by: title_path
    tree:
      title: Payment
      children:
        - title: 中文解释
"#,
    );
    let fixture = copy_fixture(MINIMAL_FIXTURE, "patch-merge-id-target.xmind");
    let workbook = fixture.path_arg();

    let dry_run = run_json(&["patch", &workbook, "--ops", &ops, "--dry-run", "--json"]);
    assert_success_envelope(&dry_run, "patch", Some(&workbook));
    assert_eq!(dry_run["result"]["summary"]["added"], 1);

    let apply = run_json(&[
        "patch", &workbook, "--ops", &ops, "--apply", "--backup", "--json",
    ]);
    assert_success_envelope(&apply, "patch", Some(&workbook));
    assert_eq!(apply["applied"], true);

    let payment = run_json(&["get", &workbook, "--node", "id:topic-payment", "--json"]);
    assert!(topic_child_titles(&payment).contains(&"中文解释".to_owned()));
    validate_workbook(&workbook);
    fixture.assert_source_unchanged();
}

#[test]
fn patch_set_apply_updates_id_selected_target() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for patch input");
    let ops = temp_file(
        temp_dir.path(),
        "set-id-target.yaml",
        r#"ops:
  - op: set
    node: id:topic-payment
    fields:
      title: Payment scope
"#,
    );
    let fixture = copy_fixture(MINIMAL_FIXTURE, "patch-set-id-target.xmind");
    let workbook = fixture.path_arg();

    let apply = run_json(&[
        "patch", &workbook, "--ops", &ops, "--apply", "--backup", "--json",
    ]);
    assert_success_envelope(&apply, "patch", Some(&workbook));
    assert_eq!(apply["applied"], true);

    let payment = run_json(&["get", &workbook, "--node", "id:topic-payment", "--json"]);
    assert_eq!(payment["result"]["topic"]["title"], "Payment scope");
    validate_workbook(&workbook);
    fixture.assert_source_unchanged();
}

#[test]
fn patch_replace_tree_generates_unique_ids_for_non_ascii_topics() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for patch input");
    let ops = temp_file(
        temp_dir.path(),
        "replace-non-ascii.yaml",
        r#"ops:
  - op: replace_tree
    node: path:/Q2
    tree:
      title: 核心概念
      children:
        - title: 底层存储
          children:
            - title: Vault
            - title: 中文解释
        - title: 笔记形式
          children:
            - title: Markdown
            - title: 另一条中文解释
"#,
    );
    let fixture = copy_fixture(MINIMAL_FIXTURE, "patch-replace-non-ascii.xmind");
    let workbook = fixture.path_arg();

    let apply = run_json(&[
        "patch", &workbook, "--ops", &ops, "--apply", "--backup", "--json",
    ]);
    assert_success_envelope(&apply, "patch", Some(&workbook));
    assert_eq!(apply["applied"], true);

    validate_workbook(&workbook);
    let root = run_json(&["tree", &workbook, "--depth", "4", "--json"]);
    assert_eq!(root["result"]["root"]["children"][0]["title"], "核心概念");
    fixture.assert_source_unchanged();
}

#[test]
fn patch_apply_rejects_structurally_invalid_candidate_without_replacing_workbook() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for patch input");
    let ops = temp_file(
        temp_dir.path(),
        "duplicate-explicit-ids.yaml",
        r#"ops:
  - op: replace_tree
    node: path:/Q2
    tree:
      id: topic-replacement
      title: Replacement
      children:
        - id: duplicate-topic
          title: First duplicate
        - id: duplicate-topic
          title: Second duplicate
"#,
    );
    let fixture = copy_fixture(MINIMAL_FIXTURE, "patch-invalid-candidate.xmind");
    let workbook = fixture.path_arg();
    let original_bytes = std::fs::read(&workbook).expect("copied workbook is readable");

    let body = run_json_error(
        &[
            "patch", &workbook, "--ops", &ops, "--apply", "--backup", "--json",
        ],
        9,
    );

    assert_eq!(body["error"]["code"], "validation_failed");
    assert_eq!(
        std::fs::read(&workbook).expect("workbook remains readable"),
        original_bytes,
        "structural validation failure must leave original workbook untouched"
    );
    validate_workbook(&workbook);
    fixture.assert_source_unchanged();
}
