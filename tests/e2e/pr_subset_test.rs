#[path = "support.rs"]
mod support;

use serde_json::Value;
use support::{
    assert_success_envelope, copy_fixture, run_human, run_human_error, run_json, run_json_error,
    run_success, temp_file, validate_workbook, write_unsupported_xmind_variant,
    write_xmind_package, DUPLICATE_SHEETS_FIXTURE, DUPLICATE_TITLES_FIXTURE, MALFORMED_FIXTURE,
    METADATA_FIXTURE, MINIMAL_FIXTURE, MULTIPLE_SHEETS_FIXTURE, REAL_APP_FIXTURE,
    TOPIC_IMAGE_FIXTURE,
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
fn real_app_fixture_decodes_tree_and_validates() {
    let inspect = run_json(&["inspect", REAL_APP_FIXTURE, "--json"]);
    assert_success_envelope(&inspect, "inspect", Some(REAL_APP_FIXTURE));
    assert_eq!(inspect["result"]["format"], "xmind-zen");

    let tree = run_json(&["tree", REAL_APP_FIXTURE, "--json"]);
    assert_success_envelope(&tree, "tree", Some(REAL_APP_FIXTURE));
    let root = &tree["result"]["root"];
    assert_eq!(root["title"], "Real App Fixture");
    let children = root["children"]
        .as_array()
        .expect("real app fixture root has children");
    assert_eq!(children.len(), 5);
    assert_eq!(children[0]["title"], "分支主题 1");

    let validation = run_json(&["validate", REAL_APP_FIXTURE, "--json"]);
    assert_success_envelope(&validation, "validate", Some(REAL_APP_FIXTURE));
    assert_eq!(validation["result"]["valid"], true);
}

#[test]
fn default_pr_subset_checks_lightweight_human_output() {
    let tree = run_human(&["tree", MINIMAL_FIXTURE, "--depth", "1"]);
    assert!(tree.contains("Roadmap"));
    assert!(tree.contains("Q2"));

    let completion = run_human(&["completion", "bash"]);
    assert!(completion.contains("_xmind"));
    assert!(completion.contains("complete -F _xmind"));

    let diff = run_human(&["diff", MINIMAL_FIXTURE]);
    assert!(diff.contains("minimal.xmind: no changes"));
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
fn read_e2e_validate_warnings_are_reported_and_strict_turns_them_into_failures() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for warning fixture");
    let workbook = temp_dir.path().join("missing-image-resource.xmind");
    write_xmind_package(
        &workbook,
        &[(
            "content.json",
            br#"[
  {
    "id": "sheet-roadmap",
    "title": "Roadmap",
    "rootTopic": {
      "id": "topic-root",
      "title": "Roadmap",
      "image": {
        "src": "xap:resources/missing.png"
      }
    }
  }
]"#,
        )],
    );
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let non_strict = run_json(&["validate", &workbook_arg, "--json"]);
    assert_eq!(non_strict["result"]["valid"], true);
    assert!(non_strict["result"]["warnings"]
        .as_array()
        .expect("validation warnings are an array")
        .iter()
        .any(|warning| warning["code"] == "missing_resource"));

    let strict = run_json_error(&["validate", &workbook_arg, "--strict", "--json"], 9);
    assert_eq!(strict["error"]["code"], "validation_failed");
    assert_eq!(strict["result"]["valid"], false);
    assert!(strict["result"]["warnings"]
        .as_array()
        .expect("strict validation warnings are an array")
        .iter()
        .any(|warning| warning["code"] == "missing_resource"));
}

#[test]
fn read_e2e_validate_reports_structural_errors() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for structural fixtures");

    let missing_content = temp_dir.path().join("missing-content.xmind");
    write_xmind_package(&missing_content, &[("metadata.json", b"{}")]);
    let missing_content_arg = missing_content.to_string_lossy().into_owned();
    let missing_content_body = run_json_error(&["validate", &missing_content_arg, "--json"], 9);
    assert_eq!(missing_content_body["error"]["code"], "validation_failed");
    assert!(missing_content_body["result"]["errors"]
        .as_array()
        .expect("missing content errors are an array")
        .iter()
        .any(|error| error["code"] == "missing_content"));

    let missing_root = temp_dir.path().join("missing-root-topic.xmind");
    write_xmind_package(
        &missing_root,
        &[(
            "content.json",
            br#"[{"id":"sheet-roadmap","title":"Roadmap"}]"#,
        )],
    );
    let missing_root_arg = missing_root.to_string_lossy().into_owned();
    let missing_root_body = run_json_error(&["validate", &missing_root_arg, "--json"], 9);
    assert!(missing_root_body["result"]["errors"]
        .as_array()
        .expect("missing root topic errors are an array")
        .iter()
        .any(|error| error["code"] == "missing_required_field"
            && error["path"] == "content.json[0].rootTopic"));

    let broken_relationship = temp_dir.path().join("broken-relationship.xmind");
    write_xmind_package(
        &broken_relationship,
        &[(
            "content.json",
            br#"[
  {
    "id": "sheet-roadmap",
    "title": "Roadmap",
    "rootTopic": {
      "id": "topic-root",
      "title": "Roadmap"
    },
    "relationships": [
      {
        "id": "relationship-1",
        "end1Id": "topic-root",
        "end2Id": "topic-missing"
      }
    ]
  }
]"#,
        )],
    );
    let broken_relationship_arg = broken_relationship.to_string_lossy().into_owned();
    let broken_relationship_body =
        run_json_error(&["validate", &broken_relationship_arg, "--json"], 9);
    assert!(broken_relationship_body["result"]["errors"]
        .as_array()
        .expect("relationship errors are an array")
        .iter()
        .any(|error| error["code"] == "broken_relationship_reference"));

    let duplicate_topic = temp_dir.path().join("duplicate-topic-id.xmind");
    write_xmind_package(
        &duplicate_topic,
        &[(
            "content.json",
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
            "id": "topic-duplicate",
            "title": "Q2",
            "children": {
              "attached": [
                {
                  "id": "topic-duplicate",
                  "title": "Payment"
                }
              ]
            }
          }
        ]
      }
    }
  }
]"#,
        )],
    );
    let duplicate_topic_arg = duplicate_topic.to_string_lossy().into_owned();
    let duplicate_topic_body =
        run_json_error(&["validate", &duplicate_topic_arg, "--strict", "--json"], 9);
    assert!(duplicate_topic_body["result"]["errors"]
        .as_array()
        .expect("duplicate topic errors are an array")
        .iter()
        .any(|error| error["code"] == "duplicate_topic_id"));
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
fn e2e_add_covers_positions_missing_path_backup_and_selector_errors() {
    let dry_run_fixture = copy_fixture(MINIMAL_FIXTURE, "add-dry-run.xmind");
    let dry_run_workbook = dry_run_fixture.path_arg();
    let dry_run = run_json(&[
        "add",
        &dry_run_workbook,
        "--parent",
        "path:/Q2",
        "--title",
        "Prep",
        "--position",
        "first",
        "--dry-run",
        "--json",
    ]);
    assert_success_envelope(&dry_run, "add", Some(&dry_run_workbook));
    assert_eq!(dry_run["dry_run"], true);
    assert_eq!(dry_run["applied"], false);
    let tree_after_dry_run = run_json(&["tree", &dry_run_workbook, "--depth", "2", "--json"]);
    assert_eq!(
        tree_after_dry_run["result"]["root"]["children"][0]["children"][0]["title"],
        "Payment"
    );
    dry_run_fixture.assert_source_unchanged();

    let apply_fixture = copy_fixture(MINIMAL_FIXTURE, "add-create-missing-backup.xmind");
    let apply_workbook = apply_fixture.path_arg();
    let apply = run_json(&[
        "add",
        &apply_workbook,
        "--parent",
        "path:/Q3/Payments",
        "--title",
        "Refunds",
        "--create-missing-path",
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&apply, "add", Some(&apply_workbook));
    assert_eq!(apply["result"]["summary"]["added"], 3);
    apply_fixture.assert_backup_matches_original(&apply);
    validate_workbook(&apply_workbook);
    let added = run_json(&[
        "get",
        &apply_workbook,
        "--node",
        "path:/Q3/Payments/Refunds",
        "--json",
    ]);
    assert_eq!(added["result"]["topic"]["title"], "Refunds");
    apply_fixture.assert_source_unchanged();

    let missing = run_json_error(
        &[
            "add",
            MINIMAL_FIXTURE,
            "--parent",
            "path:/Missing",
            "--title",
            "Refunds",
            "--dry-run",
            "--json",
        ],
        5,
    );
    assert_eq!(missing["error"]["code"], "not_found");

    let ambiguous = run_json_error(
        &[
            "add",
            DUPLICATE_TITLES_FIXTURE,
            "--parent",
            "title:Payment",
            "--title",
            "Refunds",
            "--dry-run",
            "--json",
        ],
        6,
    );
    assert_eq!(ambiguous["error"]["code"], "ambiguous_selector");
    assert!(
        ambiguous["error"]["candidates"]
            .as_array()
            .expect("ambiguous parent includes candidates")
            .len()
            >= 2
    );
}

#[test]
fn e2e_set_covers_editable_fields_clear_image_paths_and_asset_errors() {
    let metadata_fixture = copy_fixture(METADATA_FIXTURE, "set-metadata-backup.xmind");
    let metadata_workbook = metadata_fixture.path_arg();
    let set_title = run_json(&[
        "set",
        &metadata_workbook,
        "--node",
        "id:topic-payment",
        "--title",
        "Payments",
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&set_title, "set", Some(&metadata_workbook));
    assert_eq!(set_title["result"]["updated"]["changed_fields"][0], "title");
    metadata_fixture.assert_backup_matches_original(&set_title);

    let set_note = run_json(&[
        "set",
        &metadata_workbook,
        "--node",
        "id:topic-payment",
        "--note",
        "Updated payment note",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&set_note, "set", Some(&metadata_workbook));

    let set_labels = run_json(&[
        "set",
        &metadata_workbook,
        "--node",
        "id:topic-payment",
        "--set-labels",
        "Platform,Billing",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&set_labels, "set", Some(&metadata_workbook));

    let set_markers = run_json(&[
        "set",
        &metadata_workbook,
        "--node",
        "id:topic-payment",
        "--set-markers",
        "priority-2,task-done",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&set_markers, "set", Some(&metadata_workbook));

    let set_hyperlink = run_json(&[
        "set",
        &metadata_workbook,
        "--node",
        "id:topic-payment",
        "--hyperlink",
        "https://example.com/billing",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&set_hyperlink, "set", Some(&metadata_workbook));

    validate_workbook(&metadata_workbook);
    let changed = run_json(&[
        "get",
        &metadata_workbook,
        "--node",
        "id:topic-payment",
        "--include-assets",
        "--json",
    ]);
    assert_eq!(changed["result"]["topic"]["title"], "Payments");
    assert_eq!(changed["result"]["topic"]["note"], "Updated payment note");
    assert_eq!(changed["result"]["topic"]["labels"][0], "Platform");
    assert_eq!(changed["result"]["topic"]["markers"][0], "priority-2");
    assert_eq!(
        changed["result"]["topic"]["hyperlink"],
        "https://example.com/billing"
    );
    assert_eq!(
        changed["result"]["topic"]["image"]["asset_id"], "xap:resources/payment.png",
        "non-image metadata should preserve the existing image reference"
    );
    metadata_fixture.assert_source_unchanged();

    let clear_fixture = copy_fixture(METADATA_FIXTURE, "set-clear.xmind");
    let clear_workbook = clear_fixture.path_arg();
    let clear = run_json(&[
        "set",
        &clear_workbook,
        "--node",
        "id:topic-payment",
        "--clear",
        "note",
        "--clear",
        "labels",
        "--clear",
        "markers",
        "--clear",
        "hyperlink",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&clear, "set", Some(&clear_workbook));
    validate_workbook(&clear_workbook);
    let cleared = run_json(&[
        "get",
        &clear_workbook,
        "--node",
        "id:topic-payment",
        "--json",
    ]);
    assert!(cleared["result"]["topic"].get("note").is_none());
    assert!(cleared["result"]["topic"].get("hyperlink").is_none());
    assert_eq!(
        cleared["result"]["topic"]["labels"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        cleared["result"]["topic"]["markers"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    clear_fixture.assert_source_unchanged();

    let image_fixture = copy_fixture(TOPIC_IMAGE_FIXTURE, "set-image.xmind");
    let image_workbook = image_fixture.path_arg();
    let replacement_image =
        image_fixture.write_sibling("replacement.png", b"\x89PNG\r\n\x1a\nimage-bytes");
    let replace = run_json(&[
        "set",
        &image_workbook,
        "--node",
        "id:topic-payment",
        "--image",
        &replacement_image,
        "--image-alt",
        "Replacement diagram",
        "--image-title",
        "Replacement flow",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&replace, "set", Some(&image_workbook));
    validate_workbook(&image_workbook);
    let image_topic = run_json(&[
        "get",
        &image_workbook,
        "--node",
        "id:topic-payment",
        "--include-assets",
        "--json",
    ]);
    assert_eq!(
        image_topic["result"]["topic"]["image"]["asset_id"],
        "xap:resources/replacement.png"
    );
    assert_eq!(
        image_topic["result"]["topic"]["image"]["alt"],
        "Replacement diagram"
    );
    let assets = run_json(&["export", &image_workbook, "--format", "assets"]);
    let asset_ids: Vec<&str> = assets["assets"]
        .as_array()
        .expect("asset export returns an array")
        .iter()
        .map(|asset| asset["asset_id"].as_str().expect("asset id is a string"))
        .collect();
    assert!(asset_ids.contains(&"xap:resources/payment.png"));
    assert!(asset_ids.contains(&"xap:resources/replacement.png"));

    let clear_image = run_json(&[
        "set",
        &image_workbook,
        "--node",
        "id:topic-payment",
        "--clear",
        "image",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&clear_image, "set", Some(&image_workbook));
    validate_workbook(&image_workbook);
    let image_cleared = run_json(&[
        "get",
        &image_workbook,
        "--node",
        "id:topic-payment",
        "--include-assets",
        "--json",
    ]);
    assert!(image_cleared["result"]["topic"]["image"].is_null());
    image_fixture.assert_source_unchanged();

    let unsupported_fixture = copy_fixture(MINIMAL_FIXTURE, "set-unsupported-asset.xmind");
    let unsupported_workbook = unsupported_fixture.path_arg();
    let unsupported_image = unsupported_fixture.write_sibling("payment.txt", b"not an image");
    let unsupported = run_json_error(
        &[
            "set",
            &unsupported_workbook,
            "--node",
            "id:topic-payment",
            "--image",
            &unsupported_image,
            "--apply",
            "--json",
        ],
        11,
    );
    assert_eq!(unsupported["error"]["code"], "unsupported_asset_type");
    unsupported_fixture.assert_source_unchanged();
}

#[test]
fn e2e_delete_covers_subtree_children_only_promote_root_rejection_and_backup() {
    let subtree_fixture = copy_fixture(MINIMAL_FIXTURE, "delete-subtree-backup.xmind");
    let subtree_workbook = subtree_fixture.path_arg();
    let subtree = run_json(&[
        "delete",
        &subtree_workbook,
        "--node",
        "id:topic-q2",
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&subtree, "delete", Some(&subtree_workbook));
    assert_eq!(subtree["result"]["summary"]["deleted"], 2);
    subtree_fixture.assert_backup_matches_original(&subtree);
    validate_workbook(&subtree_workbook);
    let subtree_tree = run_json(&["tree", &subtree_workbook, "--depth", "2", "--json"]);
    assert_eq!(
        subtree_tree["result"]["root"]["children"]
            .as_array()
            .expect("root children is an array")
            .len(),
        0
    );
    subtree_fixture.assert_source_unchanged();

    let children_fixture = copy_fixture(MINIMAL_FIXTURE, "delete-children-only.xmind");
    let children_workbook = children_fixture.path_arg();
    let children_only = run_json(&[
        "delete",
        &children_workbook,
        "--node",
        "id:topic-q2",
        "--children-only",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&children_only, "delete", Some(&children_workbook));
    validate_workbook(&children_workbook);
    let children_tree = run_json(&["tree", &children_workbook, "--depth", "2", "--json"]);
    assert_eq!(
        children_tree["result"]["root"]["children"][0]["title"],
        "Q2"
    );
    assert_eq!(
        children_tree["result"]["root"]["children"][0]["children"]
            .as_array()
            .expect("Q2 children is an array")
            .len(),
        0
    );
    children_fixture.assert_source_unchanged();

    let promote_fixture = copy_fixture(MINIMAL_FIXTURE, "delete-promote.xmind");
    let promote_workbook = promote_fixture.path_arg();
    let promote = run_json(&[
        "delete",
        &promote_workbook,
        "--node",
        "id:topic-q2",
        "--promote-children",
        "--apply",
        "--json",
    ]);
    assert_success_envelope(&promote, "delete", Some(&promote_workbook));
    validate_workbook(&promote_workbook);
    let promoted = run_json(&[
        "get",
        &promote_workbook,
        "--node",
        "path:/Payment",
        "--json",
    ]);
    assert_eq!(promoted["result"]["topic"]["id"], "topic-payment");
    promote_fixture.assert_source_unchanged();

    let root_rejection = run_json_error(
        &[
            "delete",
            MINIMAL_FIXTURE,
            "--node",
            "root",
            "--children-only",
            "--dry-run",
            "--json",
        ],
        8,
    );
    assert_eq!(
        root_rejection["error"]["code"],
        "root_operation_not_allowed"
    );
}

#[test]
fn e2e_move_covers_positions_cycle_root_rejection_backup_and_source_integrity() {
    let move_fixture = copy_fixture(DUPLICATE_TITLES_FIXTURE, "move-position-backup.xmind");
    let move_workbook = move_fixture.path_arg();
    let moved = run_json(&[
        "move",
        &move_workbook,
        "--node",
        "id:topic-payment-q2",
        "--to",
        "root",
        "--position",
        "first",
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&moved, "move", Some(&move_workbook));
    assert_eq!(moved["result"]["moved"]["to_path"], "/Payment");
    move_fixture.assert_backup_matches_original(&moved);
    validate_workbook(&move_workbook);
    let tree = run_json(&["tree", &move_workbook, "--depth", "1", "--json"]);
    assert_eq!(
        tree["result"]["root"]["children"][0]["id"],
        "topic-payment-q2"
    );
    move_fixture.assert_source_unchanged();

    let cycle = run_json_error(
        &[
            "move",
            DUPLICATE_TITLES_FIXTURE,
            "--node",
            "id:topic-q1",
            "--to",
            "id:topic-payment-q1",
            "--dry-run",
            "--json",
        ],
        8,
    );
    assert_eq!(cycle["error"]["code"], "patch_conflict");

    let root_rejection = run_json_error(
        &[
            "move",
            DUPLICATE_TITLES_FIXTURE,
            "--node",
            "root",
            "--to",
            "id:topic-q1",
            "--dry-run",
            "--json",
        ],
        8,
    );
    assert_eq!(
        root_rejection["error"]["code"],
        "root_operation_not_allowed"
    );
}

#[test]
fn e2e_copy_covers_id_regeneration_positions_guardrails_root_rejection_and_backup() {
    let copy_fixture = copy_fixture(DUPLICATE_TITLES_FIXTURE, "copy-position-backup.xmind");
    let copy_workbook = copy_fixture.path_arg();
    let copied = run_json(&[
        "copy",
        &copy_workbook,
        "--node",
        "id:topic-payment-q1",
        "--to",
        "root",
        "--position",
        "after:id:topic-q1",
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&copied, "copy", Some(&copy_workbook));
    assert_eq!(
        copied["result"]["copied_root"]["source_id"],
        "topic-payment-q1"
    );
    assert_eq!(
        copied["result"]["copied_root"]["new_id"],
        "topic-payment-q1-copy"
    );
    copy_fixture.assert_backup_matches_original(&copied);
    validate_workbook(&copy_workbook);
    let tree = run_json(&["tree", &copy_workbook, "--depth", "1", "--json"]);
    assert_eq!(
        tree["result"]["root"]["children"][1]["id"],
        "topic-payment-q1-copy"
    );
    copy_fixture.assert_source_unchanged();

    let preserve_ids = run_json_error(
        &[
            "copy",
            DUPLICATE_TITLES_FIXTURE,
            "--node",
            "id:topic-payment-q1",
            "--to",
            "root",
            "--preserve-ids",
            "--dry-run",
            "--json",
        ],
        8,
    );
    assert_eq!(preserve_ids["error"]["code"], "patch_conflict");

    let root_rejection = run_json_error(
        &[
            "copy",
            DUPLICATE_TITLES_FIXTURE,
            "--node",
            "root",
            "--to",
            "id:topic-q1",
            "--dry-run",
            "--json",
        ],
        8,
    );
    assert_eq!(
        root_rejection["error"]["code"],
        "root_operation_not_allowed"
    );
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

    let patch_fixture = copy_fixture(MINIMAL_FIXTURE, "patch-apply-pr-subset.xmind");
    let patch_workbook = patch_fixture.path_arg();
    let patch_apply = run_json(&[
        "patch",
        &patch_workbook,
        "--ops",
        &ops,
        "--apply",
        "--backup",
        "--json",
    ]);
    assert_success_envelope(&patch_apply, "patch", Some(&patch_workbook));
    assert_eq!(patch_apply["dry_run"], false);
    assert_eq!(patch_apply["applied"], true);
    assert_eq!(patch_apply["result"]["operations"][0]["status"], "applied");
    patch_fixture.assert_backup_matches_original(&patch_apply);
    validate_workbook(&patch_workbook);
    patch_fixture.assert_source_unchanged();

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
