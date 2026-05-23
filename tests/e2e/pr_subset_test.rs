#[path = "support.rs"]
mod support;

use serde_json::Value;
use support::{
    assert_success_envelope, copy_fixture, run_human, run_human_error, run_json, run_json_error,
    run_success, temp_file, validate_workbook, write_unsupported_xmind_variant,
    DUPLICATE_SHEETS_FIXTURE, DUPLICATE_TITLES_FIXTURE, MALFORMED_FIXTURE, METADATA_FIXTURE,
    MINIMAL_FIXTURE, MULTIPLE_SHEETS_FIXTURE, TOPIC_IMAGE_FIXTURE,
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
fn read_e2e_inspect_reports_supported_resources_and_parse_errors() {
    let inspect = run_json(&["inspect", TOPIC_IMAGE_FIXTURE, "--json"]);
    assert_success_envelope(&inspect, "inspect", Some(TOPIC_IMAGE_FIXTURE));
    assert_eq!(inspect["result"]["format"], "xmind-zen");
    assert_eq!(inspect["result"]["resources_count"], 1);
    assert_eq!(inspect["result"]["capabilities"]["can_read_topics"], true);

    let human = run_human(&["inspect", TOPIC_IMAGE_FIXTURE]);
    assert!(human.contains("topic-image.xmind: 1 sheets"));

    let malformed = run_json_error(&["inspect", MALFORMED_FIXTURE, "--json"], 4);
    assert_eq!(malformed["command"], "inspect");
    assert_eq!(malformed["error"]["code"], "parse_failed");

    let human_error = run_human_error(&["inspect", MALFORMED_FIXTURE, "--no-color"], 4);
    assert!(human_error.starts_with("inspect: Workbook could not be parsed"));

    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let unsupported_path = temp_dir.path().join("legacy.xmind");
    write_unsupported_xmind_variant(&unsupported_path);
    let unsupported_arg = unsupported_path.to_string_lossy().into_owned();
    let unsupported = run_json_error(&["inspect", &unsupported_arg, "--json"], 11);
    assert_eq!(unsupported["command"], "inspect");
    assert_eq!(unsupported["error"]["code"], "unsupported_format");
}

#[test]
fn read_e2e_sheets_covers_duplicates_fields_metadata_and_human_output() {
    let duplicate = run_json(&["sheets", DUPLICATE_SHEETS_FIXTURE, "--json"]);
    assert_success_envelope(&duplicate, "sheets", Some(DUPLICATE_SHEETS_FIXTURE));
    let sheets = duplicate["result"]["sheets"]
        .as_array()
        .expect("sheets result is an array");
    assert_eq!(sheets.len(), 2);
    assert_eq!(sheets[0]["title"], "Roadmap");
    assert_eq!(sheets[1]["title"], "Roadmap");
    assert_ne!(sheets[0]["id"], sheets[1]["id"]);

    let fields = run_json(&[
        "sheets",
        MULTIPLE_SHEETS_FIXTURE,
        "--format",
        "compact-json",
        "--fields",
        "id,title,root_topic_id",
        "--json",
    ]);
    assert_success_envelope(&fields, "sheets", Some(MULTIPLE_SHEETS_FIXTURE));
    assert_eq!(fields["result"]["sheets"][1]["id"], "sheet-backlog");
    assert_eq!(
        fields["result"]["sheets"][1]["root_topic_id"],
        "topic-backlog-root"
    );
    assert!(fields["result"]["sheets"][1].get("topic_count").is_none());

    let human = run_human(&["sheets", MULTIPLE_SHEETS_FIXTURE]);
    assert!(human.contains("0: Roadmap"));
    assert!(human.contains("1: Backlog"));
}

#[test]
fn read_e2e_tree_covers_depth_fields_assets_sheet_selection_and_human_output() {
    let depth_one = run_json(&["tree", MINIMAL_FIXTURE, "--depth", "1", "--json"]);
    assert_success_envelope(&depth_one, "tree", Some(MINIMAL_FIXTURE));
    assert_eq!(depth_one["result"]["root"]["children"][0]["title"], "Q2");
    assert!(depth_one["result"]["root"]["children"][0]
        .get("children")
        .is_none());

    let assets = run_json(&[
        "tree",
        TOPIC_IMAGE_FIXTURE,
        "--include-assets",
        "--depth",
        "3",
        "--json",
    ]);
    assert_eq!(
        assets["result"]["root"]["children"][0]["children"][0]["image"]["asset_id"],
        "xap:resources/payment.png"
    );

    let selected = run_json(&[
        "tree",
        MULTIPLE_SHEETS_FIXTURE,
        "--sheet-id",
        "sheet-backlog",
        "--depth",
        "1",
        "--json",
    ]);
    assert_eq!(selected["result"]["sheet"], "Backlog");
    assert_eq!(selected["result"]["root"]["children"][0]["title"], "Ideas");

    let fields = run_json(&[
        "tree",
        MINIMAL_FIXTURE,
        "--format",
        "compact-json",
        "--fields",
        "id,title,children",
        "--depth",
        "1",
        "--json",
    ]);
    assert!(fields["result"]["root"].get("path").is_none());
    assert_eq!(fields["result"]["root"]["children"][0]["title"], "Q2");

    let human = run_human(&["tree", MINIMAL_FIXTURE, "--depth", "2"]);
    assert!(human.contains("Roadmap"));
    assert!(human.contains("Payment"));
}

#[test]
fn read_e2e_get_covers_selectors_depth_assets_and_selector_errors() {
    let by_id = run_json(&[
        "get",
        MINIMAL_FIXTURE,
        "--node",
        "id:topic-payment",
        "--json",
    ]);
    assert_success_envelope(&by_id, "get", Some(MINIMAL_FIXTURE));
    assert_eq!(by_id["result"]["topic"]["path"], "/Q2/Payment");

    let by_path = run_json(&[
        "get",
        MINIMAL_FIXTURE,
        "--node",
        "path:/Q2",
        "--depth",
        "1",
        "--json",
    ]);
    assert_eq!(
        by_path["result"]["topic"]["children"][0]["title"],
        "Payment"
    );

    let by_title = run_json(&["get", MINIMAL_FIXTURE, "--node", "title:Payment", "--json"]);
    assert_eq!(by_title["result"]["topic"]["id"], "topic-payment");

    let by_query = run_json(&[
        "get",
        MINIMAL_FIXTURE,
        "--node",
        "query:title = \"Payment\"",
        "--json",
    ]);
    assert_eq!(by_query["result"]["topic"]["id"], "topic-payment");

    let assets = run_json(&[
        "get",
        TOPIC_IMAGE_FIXTURE,
        "--node",
        "id:topic-payment",
        "--include-assets",
        "--json",
    ]);
    assert_eq!(
        assets["result"]["topic"]["image"]["asset_id"],
        "xap:resources/payment.png"
    );

    let selected = run_json(&[
        "get",
        MULTIPLE_SHEETS_FIXTURE,
        "--sheet",
        "Backlog",
        "--node",
        "root",
        "--depth",
        "1",
        "--json",
    ]);
    assert_eq!(selected["result"]["topic"]["children"][0]["title"], "Ideas");

    let missing = run_json_error(
        &["get", MINIMAL_FIXTURE, "--node", "path:/Missing", "--json"],
        5,
    );
    assert_eq!(missing["error"]["code"], "not_found");

    let ambiguous = run_json_error(
        &[
            "get",
            DUPLICATE_TITLES_FIXTURE,
            "--node",
            "title:Payment",
            "--json",
        ],
        6,
    );
    assert_eq!(ambiguous["error"]["code"], "ambiguous_selector");
    assert!(
        ambiguous["error"]["candidates"]
            .as_array()
            .expect("ambiguous get returns candidates")
            .len()
            >= 2
    );
}

#[test]
fn read_e2e_find_covers_match_modes_pagination_and_empty_results() {
    let exact = run_json(&["find", MINIMAL_FIXTURE, "--title", "Payment", "--json"]);
    assert_success_envelope(&exact, "find", Some(MINIMAL_FIXTURE));
    assert_eq!(exact["result"]["matches"][0]["id"], "topic-payment");

    let title_contains = run_json(&["find", MINIMAL_FIXTURE, "--title-contains", "Pay", "--json"]);
    assert_eq!(
        title_contains["result"]["matches"][0]["path"],
        "/Q2/Payment"
    );

    let content_contains = run_json(&["find", METADATA_FIXTURE, "--contains", "refund", "--json"]);
    assert_eq!(
        content_contains["result"]["matches"][0]["id"],
        "topic-payment"
    );

    let query = run_json(&[
        "find",
        MINIMAL_FIXTURE,
        "--query",
        "title != \"Payment\"",
        "--offset",
        "1",
        "--limit",
        "1",
        "--json",
    ]);
    let paged = query["result"]["matches"]
        .as_array()
        .expect("find matches are an array");
    assert_eq!(paged.len(), 1);
    assert_eq!(paged[0]["id"], "topic-q2");

    let none = run_json(&["find", MINIMAL_FIXTURE, "--title", "Missing", "--json"]);
    assert!(none["result"]["matches"]
        .as_array()
        .expect("find matches are an array")
        .is_empty());
}

#[test]
fn read_e2e_validate_covers_valid_strict_parse_error_and_human_output() {
    let valid = run_json(&["validate", MINIMAL_FIXTURE, "--json"]);
    assert_success_envelope(&valid, "validate", Some(MINIMAL_FIXTURE));
    assert_eq!(valid["result"]["valid"], true);
    assert_eq!(valid["result"]["warnings"], Value::Array(vec![]));
    assert_eq!(valid["result"]["errors"], Value::Array(vec![]));

    let strict = run_json(&["validate", MINIMAL_FIXTURE, "--strict", "--json"]);
    assert_eq!(strict["result"]["valid"], true);

    let human = run_human(&["validate", MINIMAL_FIXTURE]);
    assert!(human.contains("minimal.xmind: valid"));

    let malformed = run_json_error(&["validate", MALFORMED_FIXTURE, "--json"], 4);
    assert_eq!(malformed["error"]["code"], "parse_failed");
}

#[test]
fn read_e2e_read_commands_return_invalid_usage_errors() {
    let unknown_field = run_json_error(
        &[
            "inspect",
            MINIMAL_FIXTURE,
            "--fields",
            "unknown_field",
            "--json",
        ],
        2,
    );
    assert_eq!(unknown_field["error"]["code"], "invalid_usage");

    let missing_node = run_json_error(&["get", MINIMAL_FIXTURE, "--json"], 2);
    assert_eq!(missing_node["command"], "get");
    assert_eq!(missing_node["error"]["code"], "invalid_usage");

    let missing_find_criterion = run_json_error(&["find", MINIMAL_FIXTURE, "--json"], 2);
    assert_eq!(missing_find_criterion["command"], "find");
    assert_eq!(missing_find_criterion["error"]["code"], "invalid_usage");
}

#[test]
#[ignore = "pending issue #21: real validation warnings are not emitted yet"]
fn read_e2e_validate_warnings_are_reported_and_strict_turns_them_into_failures() {
    unimplemented!("issue #21 should add a warning-producing fixture and strict failure contract");
}

#[test]
#[ignore = "pending issue #21: structural validation errors are not implemented yet"]
fn read_e2e_validate_reports_structural_errors() {
    unimplemented!(
        "issue #21 should add structural validation fixtures and validation_failed output"
    );
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
