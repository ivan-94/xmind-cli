#[path = "support.rs"]
mod support;

use serde_json::Value;
use support::{
    assert_success_envelope, copy_fixture, run_human, run_json, run_json_error, run_success,
    temp_file, validate_workbook, DUPLICATE_TITLES_FIXTURE, MINIMAL_FIXTURE,
};

type MutationAssertion = fn(&Value);

struct MutationCase {
    source: &'static str,
    file_name: &'static str,
    args: Vec<&'static str>,
    assertion: MutationAssertion,
}

#[test]
fn default_pr_subset_covers_read_command_success_paths() {
    let cases: [(&str, &[&str]); 6] = [
        ("inspect", &["inspect", MINIMAL_FIXTURE, "--json"]),
        ("sheets", &["sheets", MINIMAL_FIXTURE, "--json"]),
        ("tree", &["tree", MINIMAL_FIXTURE, "--depth", "1", "--json"]),
        (
            "get",
            &[
                "get",
                MINIMAL_FIXTURE,
                "--node",
                "path:/Q2",
                "--depth",
                "1",
                "--json",
            ],
        ),
        (
            "find",
            &["find", MINIMAL_FIXTURE, "--title", "Payment", "--json"],
        ),
        ("validate", &["validate", MINIMAL_FIXTURE, "--json"]),
    ];

    for (command, args) in cases {
        let body = run_json(args);
        assert_success_envelope(&body, command, Some(MINIMAL_FIXTURE));
    }

    run_success(&["diff", MINIMAL_FIXTURE, "--json"]);
}

#[test]
fn default_pr_subset_checks_lightweight_human_output() {
    let tree = run_human(&["tree", MINIMAL_FIXTURE, "--depth", "1"]);
    assert!(tree.contains("Roadmap"));
    assert!(tree.contains("Q2"));

    let completion = run_human(&["completion", "bash"]);
    assert!(completion.contains("_xmind"));
    assert!(completion.contains("complete -F _xmind"));
}

#[test]
fn default_pr_subset_covers_representative_json_error_families() {
    let missing = run_json_error(
        &["inspect", "tests/fixtures/xmind/missing.xmind", "--json"],
        3,
    );
    assert_eq!(missing["ok"], false);
    assert_eq!(missing["command"], "inspect");
    assert_eq!(missing["error"]["code"], "file_not_found");
    assert!(missing["error"]["suggested_fix"].is_string());

    let ambiguous = run_json_error(
        &[
            "tree",
            "tests/fixtures/xmind/duplicate-sheets.xmind",
            "--sheet",
            "Roadmap",
            "--json",
        ],
        6,
    );
    assert_eq!(ambiguous["ok"], false);
    assert_eq!(ambiguous["command"], "tree");
    assert_eq!(ambiguous["error"]["code"], "ambiguous_sheet");
    assert!(
        ambiguous["error"]["candidates"]
            .as_array()
            .expect("ambiguous errors include candidates")
            .len()
            >= 2
    );

    let invalid_usage = run_json_error(&["add", MINIMAL_FIXTURE, "--json"], 2);
    assert_eq!(invalid_usage["ok"], false);
    assert_eq!(invalid_usage["error"]["code"], "invalid_usage");
}

#[test]
fn default_pr_subset_applies_topic_mutations_to_temp_copies_then_validates() {
    let mutation_cases = [
        MutationCase {
            source: MINIMAL_FIXTURE,
            file_name: "add.xmind",
            args: vec![
                "add", "--parent", "path:/Q2", "--title", "Refund", "--apply", "--json",
            ],
            assertion: |body| {
                assert_eq!(body["command"], "add");
                assert_eq!(body["applied"], true);
                assert_eq!(body["result"]["created"]["path"], "/Q2/Refund");
            },
        },
        MutationCase {
            source: MINIMAL_FIXTURE,
            file_name: "set.xmind",
            args: vec![
                "set",
                "--node",
                "id:topic-payment",
                "--title",
                "Payments",
                "--apply",
                "--json",
            ],
            assertion: |body| {
                assert_eq!(body["command"], "set");
                assert_eq!(body["applied"], true);
                assert_eq!(body["result"]["updated"]["new_path"], "/Q2/Payments");
            },
        },
        MutationCase {
            source: MINIMAL_FIXTURE,
            file_name: "delete.xmind",
            args: vec!["delete", "--node", "path:/Q2/Payment", "--apply", "--json"],
            assertion: |body| {
                assert_eq!(body["command"], "delete");
                assert_eq!(body["applied"], true);
                assert_eq!(body["result"]["summary"]["deleted"], 1);
            },
        },
        MutationCase {
            source: DUPLICATE_TITLES_FIXTURE,
            file_name: "move.xmind",
            args: vec![
                "move",
                "--node",
                "id:topic-payment-q1",
                "--to",
                "root",
                "--apply",
                "--json",
            ],
            assertion: |body| {
                assert_eq!(body["command"], "move");
                assert_eq!(body["applied"], true);
                assert_eq!(body["result"]["moved"]["to_path"], "/Payment");
            },
        },
        MutationCase {
            source: DUPLICATE_TITLES_FIXTURE,
            file_name: "copy.xmind",
            args: vec![
                "copy",
                "--node",
                "id:topic-payment-q1",
                "--to",
                "root",
                "--apply",
                "--json",
            ],
            assertion: |body| {
                assert_eq!(body["command"], "copy");
                assert_eq!(body["applied"], true);
                assert_eq!(body["result"]["copied_root"]["path"], "/Payment");
            },
        },
    ];

    for case in mutation_cases {
        let fixture = copy_fixture(case.source, case.file_name);
        let workbook = fixture.path_arg();
        let mut args: Vec<String> = case.args.iter().map(|arg| (*arg).to_owned()).collect();
        args.insert(1, workbook.clone());
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

        let body = run_json(&arg_refs);
        (case.assertion)(&body);
        validate_workbook(&workbook);
        fixture.assert_source_unchanged();
    }
}

#[test]
fn default_pr_subset_covers_batch_exchange_backup_restore_paths() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for E2E batch tests");

    let add_tree = run_json(&[
        "add-tree",
        MINIMAL_FIXTURE,
        "--parent",
        "path:/Q2",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--dry-run",
        "--json",
    ]);
    assert_success_envelope(&add_tree, "add-tree", Some(MINIMAL_FIXTURE));
    assert_eq!(add_tree["dry_run"], true);
    assert_eq!(add_tree["result"]["summary"]["added"], 9);

    let ops = temp_file(
        temp_dir.path(),
        "patch-add.yaml",
        "ops:\n  - op: add\n    parent: path:/Q2\n    title: Refund\n",
    );
    let patch_dry_run = run_json(&[
        "patch",
        MINIMAL_FIXTURE,
        "--ops",
        &ops,
        "--dry-run",
        "--json",
    ]);
    assert_success_envelope(&patch_dry_run, "patch", Some(MINIMAL_FIXTURE));
    assert_eq!(patch_dry_run["result"]["operations"][0]["op"], "add");

    let patch_apply = run_json_error(
        &["patch", MINIMAL_FIXTURE, "--ops", &ops, "--apply", "--json"],
        2,
    );
    assert_eq!(patch_apply["command"], "patch");
    assert_eq!(patch_apply["error"]["code"], "invalid_usage");

    let imported = temp_dir.path().join("imported.xmind");
    let imported_arg = imported.to_string_lossy().into_owned();
    let import_body = run_json(&[
        "import",
        "--input",
        "docs/examples/simple-tree.yaml",
        "--output",
        &imported_arg,
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&import_body, "import", Some(&imported_arg));
    validate_workbook(&imported_arg);

    let exported = temp_dir.path().join("exported.md");
    let exported_arg = exported.to_string_lossy().into_owned();
    let export_body = run_json(&[
        "export",
        &imported_arg,
        "--format",
        "markdown",
        "--output",
        &exported_arg,
        "--json",
    ]);
    assert_success_envelope(&export_body, "export", Some(&imported_arg));
    assert!(std::fs::read_to_string(&exported)
        .expect("markdown export is written")
        .contains("支付能力"));

    let fixture = copy_fixture(MINIMAL_FIXTURE, "backup-restore.xmind");
    let workbook = fixture.path_arg();
    let backup = run_json(&["backup", &workbook, "--json"]);
    assert_success_envelope(&backup, "backup", Some(&workbook));

    std::fs::write(&workbook, b"corrupt current workbook").expect("workbook can be corrupted");
    let restore = run_json(&["restore", &workbook, "--apply", "--json"]);
    assert_success_envelope(&restore, "restore", Some(&workbook));
    validate_workbook(&workbook);
    fixture.assert_source_unchanged();
}
