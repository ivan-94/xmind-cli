#[path = "support.rs"]
#[allow(dead_code)]
mod support;

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use support::{
    assert_success_envelope, copy_fixture, run_human, run_json, run_json_error, temp_file,
    validate_workbook, MINIMAL_FIXTURE, TOPIC_IMAGE_FIXTURE,
};

fn command_output(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("xmind")
        .expect("xmind binary is built for E2E tests")
        .args(args)
        .output()
        .expect("xmind command runs")
}

fn titles_in_workbook(workbook: &str) -> Vec<String> {
    let body = run_json(&["tree", workbook, "--depth", "6", "--json"]);
    let mut titles = Vec::new();
    collect_titles(&body["result"]["root"], &mut titles);
    titles
}

fn collect_titles(topic: &Value, titles: &mut Vec<String>) {
    if let Some(title) = topic["title"].as_str() {
        titles.push(title.to_owned());
    }
    if let Some(children) = topic["children"].as_array() {
        for child in children {
            collect_titles(child, titles);
        }
    }
}

fn operation_names(body: &Value) -> Vec<String> {
    body["result"]["operations"]
        .as_array()
        .expect("patch result includes operations")
        .iter()
        .map(|operation| {
            operation["op"]
                .as_str()
                .expect("operation has canonical name")
                .to_owned()
        })
        .collect()
}

fn write_patch_all_ops(dir: &Path) -> String {
    temp_file(
        dir,
        "all-ops.yaml",
        r#"
ops:
  - op: assert_exists
    node: path:/Q2
  - op: assert_not_exists
    node: path:/Q2/Refund
  - op: ensure_path
    path: /Q3
  - op: add
    parent: path:/Q2
    title: Refund
  - op: add_tree
    parent: path:/Q2
    tree:
      title: Checkout
      children:
        - title: Cards
  - op: set
    node: path:/Q2/Payment
    fields:
      title: Payments
      note: Payment scope
  - op: copy
    node: path:/Q2/Refund
    to: path:/Q3
  - op: move
    node: path:/Q2/Checkout
    to: path:/Q3
  - op: merge_tree
    target: path:/Q3/Checkout
    match_by: title_path
    tree:
      title: Checkout
      children:
        - title: Cards
        - title: Wallets
  - op: replace_tree
    node: path:/Q3/Refund
    tree:
      title: Support
  - op: sort_children
    node: path:/Q3
    by: title
    order: desc
  - op: set_tree_metadata
    node: path:/Q3
    recursive: true
    add_labels:
      - Batch
  - op: delete
    node: path:/Q2/Payments
"#,
    )
}

#[test]
fn add_tree_e2e_covers_input_formats_dry_run_apply_backup_and_invalid_input() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for add-tree inputs");

    let dry_run_fixture = copy_fixture(MINIMAL_FIXTURE, "add-tree-yaml-dry-run.xmind");
    let dry_run_workbook = dry_run_fixture.path_arg();
    let dry_run = run_json(&[
        "add-tree",
        &dry_run_workbook,
        "--parent",
        "path:/Q2",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--dry-run",
        "--json",
    ]);
    assert_success_envelope(&dry_run, "add-tree", Some(&dry_run_workbook));
    assert_eq!(dry_run["dry_run"], true);
    assert_eq!(dry_run["applied"], false);
    assert_eq!(dry_run["result"]["summary"]["added"], 9);
    assert!(!titles_in_workbook(&dry_run_workbook).contains(&"支付能力".to_owned()));
    dry_run_fixture.assert_source_unchanged();

    let apply_fixture = copy_fixture(MINIMAL_FIXTURE, "add-tree-yaml-apply.xmind");
    let apply_workbook = apply_fixture.path_arg();
    let apply = run_json(&[
        "add-tree",
        &apply_workbook,
        "--parent",
        "path:/Q2",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&apply, "add-tree", Some(&apply_workbook));
    assert_eq!(apply["dry_run"], false);
    assert_eq!(apply["applied"], true);
    assert_eq!(apply["result"]["summary"], dry_run["result"]["summary"]);
    apply_fixture.assert_backup_matches_original(&apply);
    validate_workbook(&apply_workbook);
    assert!(titles_in_workbook(&apply_workbook).contains(&"支付能力".to_owned()));
    apply_fixture.assert_source_unchanged();

    let json_fixture = copy_fixture(MINIMAL_FIXTURE, "add-tree-json.xmind");
    let json_workbook = json_fixture.path_arg();
    let json = run_json(&[
        "add-tree",
        &json_workbook,
        "--parent",
        "path:/Q2",
        "--input",
        "docs/examples/simple-tree.json",
        "--dry-run",
        "--json",
    ]);
    assert_success_envelope(&json, "add-tree", Some(&json_workbook));
    assert_eq!(json["result"]["created_root"]["path"], "/Q2/支付能力");
    json_fixture.assert_source_unchanged();

    let markdown = temp_file(
        temp_dir.path(),
        "outline.md",
        "# Growth\n\n## Activation\n\n### Onboarding\n\n## Retention\n",
    );
    let markdown_fixture = copy_fixture(MINIMAL_FIXTURE, "add-tree-markdown.xmind");
    let markdown_workbook = markdown_fixture.path_arg();
    let markdown_body = run_json(&[
        "add-tree",
        &markdown_workbook,
        "--parent",
        "path:/Q2",
        "--from-markdown",
        &markdown,
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&markdown_body, "add-tree", Some(&markdown_workbook));
    assert_eq!(
        markdown_body["result"]["created_root"]["path"],
        "/Q2/Growth"
    );
    validate_workbook(&markdown_workbook);
    markdown_fixture.assert_source_unchanged();

    let invalid = temp_file(temp_dir.path(), "invalid.yaml", "title: ''\n");
    let invalid_body = run_json_error(
        &[
            "add-tree",
            MINIMAL_FIXTURE,
            "--parent",
            "path:/Q2",
            "--input",
            &invalid,
            "--dry-run",
            "--json",
        ],
        7,
    );
    assert_eq!(invalid_body["error"]["code"], "invalid_tree_input");
}

#[test]
fn patch_e2e_covers_every_operation_aliases_parity_errors_rollback_and_backup() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for patch inputs");
    let all_ops = write_patch_all_ops(temp_dir.path());
    let expected_ops = vec![
        "assert_exists",
        "assert_not_exists",
        "ensure_path",
        "add",
        "add_tree",
        "set",
        "copy",
        "move",
        "merge_tree",
        "replace_tree",
        "sort_children",
        "set_tree_metadata",
        "delete",
    ];

    let dry_fixture = copy_fixture(MINIMAL_FIXTURE, "patch-all-ops-dry-run.xmind");
    let dry_workbook = dry_fixture.path_arg();
    let dry_run = run_json(&[
        "patch",
        &dry_workbook,
        "--ops",
        &all_ops,
        "--dry-run",
        "--json",
    ]);
    assert_success_envelope(&dry_run, "patch", Some(&dry_workbook));
    assert_eq!(dry_run["dry_run"], true);
    assert_eq!(operation_names(&dry_run), expected_ops);
    assert!(!titles_in_workbook(&dry_workbook).contains(&"Support".to_owned()));
    dry_fixture.assert_source_unchanged();

    let apply_fixture = copy_fixture(MINIMAL_FIXTURE, "patch-all-ops-apply.xmind");
    let apply_workbook = apply_fixture.path_arg();
    let apply = run_json(&[
        "patch",
        &apply_workbook,
        "--ops",
        &all_ops,
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&apply, "patch", Some(&apply_workbook));
    assert_eq!(apply["dry_run"], false);
    assert_eq!(apply["applied"], true);
    assert_eq!(operation_names(&apply), expected_ops);
    assert_eq!(apply["result"]["summary"], dry_run["result"]["summary"]);
    apply_fixture.assert_backup_matches_original(&apply);
    validate_workbook(&apply_workbook);
    let titles = titles_in_workbook(&apply_workbook);
    assert!(titles.contains(&"Support".to_owned()));
    assert!(titles.contains(&"Wallets".to_owned()));
    assert!(!titles.contains(&"Payments".to_owned()));
    apply_fixture.assert_source_unchanged();

    let aliases = temp_file(
        temp_dir.path(),
        "aliases.yaml",
        r#"
ops:
  - op: ensure_path
    path: /Q3
  - op: clone_tree
    node: path:/Q2/Payment
    to: path:/Q3
  - op: move_tree
    node: path:/Q3/Payment
    to: path:/Q2
  - op: delete_tree
    node: path:/Q2/Payment
"#,
    );
    let alias_body = run_json(&[
        "patch",
        MINIMAL_FIXTURE,
        "--ops",
        &aliases,
        "--dry-run",
        "--json",
    ]);
    assert_eq!(
        operation_names(&alias_body),
        vec!["ensure_path", "copy", "move", "delete"]
    );

    let indexed_error = temp_file(
        temp_dir.path(),
        "indexed-error.yaml",
        "ops:\n  - op: assert_exists\n    node: path:/Q2\n  - op: delete\n",
    );
    let error_body = run_json_error(
        &[
            "patch",
            MINIMAL_FIXTURE,
            "--ops",
            &indexed_error,
            "--dry-run",
            "--json",
        ],
        7,
    );
    assert_eq!(error_body["error"]["code"], "invalid_patch");
    assert_eq!(error_body["error"]["operation_index"], 1);
    assert_eq!(error_body["error"]["field_path"], "ops[1].node");

    let rollback_ops = temp_file(
        temp_dir.path(),
        "rollback.yaml",
        "ops:\n  - op: add\n    parent: path:/Q2\n    title: Should Roll Back\n  - op: set\n    node: path:/missing\n    fields:\n      title: Unreachable\n",
    );
    let rollback_fixture = copy_fixture(MINIMAL_FIXTURE, "patch-rollback.xmind");
    let rollback_workbook = rollback_fixture.path_arg();
    let rollback = run_json_error(
        &[
            "patch",
            &rollback_workbook,
            "--ops",
            &rollback_ops,
            "--apply",
            "--json",
        ],
        5,
    );
    assert_eq!(rollback["error"]["code"], "not_found");
    assert_eq!(rollback["error"]["operation_index"], 1);
    assert!(!titles_in_workbook(&rollback_workbook).contains(&"Should Roll Back".to_owned()));
    rollback_fixture.assert_source_unchanged();
}

#[test]
fn import_export_e2e_covers_formats_output_into_overwrite_assets_and_dry_run_safety() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for import/export");

    let yaml_output = temp_dir.path().join("yaml-output.xmind");
    let yaml_output_arg = yaml_output.to_string_lossy().into_owned();
    let yaml_import = run_json(&[
        "import",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--output",
        &yaml_output_arg,
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&yaml_import, "import", Some(&yaml_output_arg));
    validate_workbook(&yaml_output_arg);

    let json_dry_run = temp_dir.path().join("json-dry-run.xmind");
    let json_dry_run_arg = json_dry_run.to_string_lossy().into_owned();
    let json_import = run_json(&[
        "import",
        "--input",
        "docs/examples/simple-tree.json",
        "--output",
        &json_dry_run_arg,
        "--dry-run",
        "--json",
    ]);
    assert_success_envelope(&json_import, "import", Some(&json_dry_run_arg));
    assert_eq!(json_import["dry_run"], true);
    assert!(
        !json_dry_run.exists(),
        "import --output --dry-run must not create a workbook"
    );

    let markdown = temp_file(temp_dir.path(), "import.md", "# Imported\n\n## Child\n");
    let markdown_output = temp_dir.path().join("markdown-output.xmind");
    let markdown_output_arg = markdown_output.to_string_lossy().into_owned();
    let markdown_import = run_json(&[
        "import",
        "--input",
        &markdown,
        "--output",
        &markdown_output_arg,
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&markdown_import, "import", Some(&markdown_output_arg));
    assert!(titles_in_workbook(&markdown_output_arg).contains(&"Imported".to_owned()));

    let overwrite = run_json_error(
        &[
            "import",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--output",
            &markdown_output_arg,
            "--apply",
            "--json",
        ],
        10,
    );
    assert_eq!(overwrite["error"]["code"], "write_failed");
    let overwritten = run_json(&[
        "import",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--output",
        &markdown_output_arg,
        "--overwrite",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&overwritten, "import", Some(&markdown_output_arg));

    let into_fixture = copy_fixture(MINIMAL_FIXTURE, "import-into-backup.xmind");
    let into_workbook = into_fixture.path_arg();
    let into = run_json(&[
        "import",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--into",
        &into_workbook,
        "--parent",
        "path:/Q2",
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&into, "import", Some(&into_workbook));
    into_fixture.assert_backup_matches_original(&into);
    validate_workbook(&into_workbook);
    into_fixture.assert_source_unchanged();

    let raw_json = run_json(&["export", &yaml_output_arg, "--format", "json"]);
    assert_eq!(raw_json["root"]["title"], "支付能力");
    assert!(
        raw_json.get("ok").is_none(),
        "raw JSON export is not an envelope"
    );

    let markdown_stdout = run_human(&["export", MINIMAL_FIXTURE, "--format", "markdown"]);
    assert!(markdown_stdout.contains("# Roadmap"));
    let outline_stdout = run_human(&["export", MINIMAL_FIXTURE, "--format", "outline"]);
    assert!(outline_stdout.contains("  Q2"));
    let text_stdout = run_human(&["export", MINIMAL_FIXTURE, "--format", "text"]);
    assert!(text_stdout.contains("Payment"));

    let assets_stdout = run_json(&["export", TOPIC_IMAGE_FIXTURE, "--format", "assets"]);
    assert_eq!(assets_stdout["format"], "assets");
    assert_eq!(
        assets_stdout["assets"][0]["asset_id"],
        "xap:resources/payment.png"
    );

    let markdown_output_file = temp_dir.path().join("exported.md");
    let markdown_output_arg = markdown_output_file.to_string_lossy().into_owned();
    let output_body = run_json(&[
        "export",
        MINIMAL_FIXTURE,
        "--format",
        "markdown",
        "--output",
        &markdown_output_arg,
        "--json",
    ]);
    assert_success_envelope(&output_body, "export", Some(MINIMAL_FIXTURE));
    assert!(fs::read_to_string(&markdown_output_file)
        .expect("markdown export file is readable")
        .contains("# Roadmap"));

    let reject_existing = run_json_error(
        &[
            "export",
            MINIMAL_FIXTURE,
            "--format",
            "markdown",
            "--output",
            &markdown_output_arg,
            "--json",
        ],
        10,
    );
    assert_eq!(reject_existing["error"]["code"], "write_failed");
    let overwrite_existing = run_json(&[
        "export",
        MINIMAL_FIXTURE,
        "--format",
        "markdown",
        "--output",
        &markdown_output_arg,
        "--overwrite",
        "--json",
    ]);
    assert_success_envelope(&overwrite_existing, "export", Some(MINIMAL_FIXTURE));

    let assets_dir = temp_dir.path().join("assets");
    let assets_dir_arg = assets_dir.to_string_lossy().into_owned();
    let assets_output = run_json(&[
        "export",
        TOPIC_IMAGE_FIXTURE,
        "--format",
        "assets",
        "--output",
        &assets_dir_arg,
        "--json",
    ]);
    assert_success_envelope(&assets_output, "export", Some(TOPIC_IMAGE_FIXTURE));
    assert_eq!(
        fs::read(assets_dir.join("resources/payment.png")).expect("asset export is readable"),
        b"png-bytes"
    );
}

#[test]
fn backup_restore_and_completion_e2e_cover_recovery_and_shell_integration() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for recovery");

    let default_fixture = copy_fixture(MINIMAL_FIXTURE, "backup-default.xmind");
    let default_workbook = default_fixture.path_arg();
    let default_backup = run_json(&["backup", &default_workbook, "--json"]);
    assert_success_envelope(&default_backup, "backup", Some(&default_workbook));
    let default_backup_path = default_backup["result"]["backup_path"]
        .as_str()
        .expect("backup returns a path");
    assert!(default_backup_path.contains(".xmind-backups"));
    assert_eq!(
        fs::read(default_backup_path).expect("default backup is readable"),
        fs::read(&default_workbook).expect("workbook is readable")
    );
    default_fixture.assert_source_unchanged();

    let custom_fixture = copy_fixture(MINIMAL_FIXTURE, "backup-custom.xmind");
    let custom_workbook = custom_fixture.path_arg();
    let custom_dir = temp_dir.path().join("custom-backups");
    let custom_dir_arg = custom_dir.to_string_lossy().into_owned();
    let custom_backup = run_json(&[
        "backup",
        &custom_workbook,
        "--backup-dir",
        &custom_dir_arg,
        "--json",
    ]);
    assert_success_envelope(&custom_backup, "backup", Some(&custom_workbook));
    assert!(custom_backup["result"]["backup_path"]
        .as_str()
        .expect("custom backup path is returned")
        .starts_with(&custom_dir_arg));
    custom_fixture.assert_source_unchanged();

    let invalid_backup_dir = temp_dir.path().join("not-a-dir");
    fs::write(&invalid_backup_dir, b"file blocks backup dir").expect("blocking file is written");
    let invalid_backup_dir_arg = invalid_backup_dir.to_string_lossy().into_owned();
    let invalid_backup = run_json_error(
        &[
            "backup",
            MINIMAL_FIXTURE,
            "--backup-dir",
            &invalid_backup_dir_arg,
            "--json",
        ],
        10,
    );
    assert_eq!(invalid_backup["error"]["code"], "write_failed");

    let restore_fixture = copy_fixture(MINIMAL_FIXTURE, "restore-latest.xmind");
    let restore_workbook = restore_fixture.path_arg();
    let backup_dir = Path::new(&restore_workbook)
        .parent()
        .expect("restore workbook has parent")
        .join(".xmind-backups");
    fs::create_dir_all(&backup_dir).expect("restore backup dir is created");
    let older_backup = backup_dir.join("restore-latest.1.xmind");
    let latest_backup = backup_dir.join("restore-latest.2.xmind");
    fs::copy(MINIMAL_FIXTURE, &older_backup).expect("older backup is copied");
    fs::copy(TOPIC_IMAGE_FIXTURE, &latest_backup).expect("latest backup is copied");
    let corrupt_current = b"corrupt current workbook".to_vec();
    fs::write(&restore_workbook, &corrupt_current).expect("current workbook is corrupted");

    let dry_run = run_json(&["restore", &restore_workbook, "--dry-run", "--json"]);
    assert_success_envelope(&dry_run, "restore", Some(&restore_workbook));
    assert_eq!(dry_run["dry_run"], true);
    assert_eq!(dry_run["applied"], false);
    assert_eq!(
        dry_run["result"]["restored_from"].as_str(),
        Some(latest_backup.to_string_lossy().as_ref())
    );
    assert_eq!(
        fs::read(&restore_workbook).expect("dry-run leaves current workbook untouched"),
        corrupt_current
    );

    let apply = run_json(&[
        "restore",
        &restore_workbook,
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&apply, "restore", Some(&restore_workbook));
    assert_eq!(apply["applied"], true);
    assert_eq!(
        apply["result"]["restored_from"].as_str(),
        Some(latest_backup.to_string_lossy().as_ref())
    );
    assert_eq!(
        fs::read(
            apply["result"]["backup_path"]
                .as_str()
                .expect("restore --backup returns backup_path")
        )
        .expect("pre-restore backup is readable"),
        corrupt_current
    );
    validate_workbook(&restore_workbook);
    assert!(titles_in_workbook(&restore_workbook).contains(&"Payment".to_owned()));
    restore_fixture.assert_source_unchanged();

    let invalid_restore_fixture = copy_fixture(MINIMAL_FIXTURE, "restore-invalid.xmind");
    let invalid_restore_workbook = invalid_restore_fixture.path_arg();
    let invalid_restore_dir = Path::new(&invalid_restore_workbook)
        .parent()
        .expect("invalid restore workbook has parent")
        .join(".xmind-backups");
    fs::create_dir_all(&invalid_restore_dir).expect("invalid restore backup dir is created");
    fs::write(
        invalid_restore_dir.join("restore-invalid.1.xmind"),
        b"invalid backup bytes",
    )
    .expect("invalid backup is written");
    let invalid_restore = run_json_error(
        &["restore", &invalid_restore_workbook, "--apply", "--json"],
        9,
    );
    assert_eq!(invalid_restore["error"]["code"], "validation_failed");
    validate_workbook(&invalid_restore_workbook);
    invalid_restore_fixture.assert_source_unchanged();

    for shell in ["bash", "elvish", "fish", "powershell", "zsh"] {
        let output = command_output(&["completion", shell]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "completion {shell} should succeed"
        );
        assert!(
            output.stderr.is_empty(),
            "completion must not emit stderr diagnostics: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !output.stdout.is_empty(),
            "completion {shell} writes stdout"
        );
        assert!(
            serde_json::from_slice::<Value>(&output.stdout).is_err(),
            "completion {shell} stdout is a shell script, not JSON"
        );
    }
}
