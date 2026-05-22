use assert_cmd::Command;
use serde_json::Value;
use std::fs;

#[test]
fn set_title_dry_run_reports_updated_topic_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "id:topic-payment",
            "--title",
            "Payments",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["updated"]["id"], "topic-payment");
    assert_eq!(body["result"]["updated"]["old_path"], "/Q2/Payment");
    assert_eq!(body["result"]["updated"]["new_path"], "/Q2/Payments");
    assert_eq!(body["result"]["updated"]["changed_fields"][0], "title");
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payments");

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
fn set_title_dry_run_human_output_includes_outline_diff() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "path:/Q2/Payment",
            "--title",
            "Payments",
            "--dry-run",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "~ /Q2/Payments title\n"
    );
}

#[test]
fn set_note_dry_run_reports_updated_topic_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "id:topic-payment",
            "--note",
            "Refund details",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["updated"]["id"], "topic-payment");
    assert_eq!(body["result"]["updated"]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["updated"]["changed_fields"][0], "note");
    assert_eq!(body["result"]["summary"]["updated"], 1);
    assert_eq!(body["result"]["diff"][0]["event"], "updated");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/Payment");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "id:topic-payment",
            "--json",
        ])
        .output()
        .expect("get command runs after dry run");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert!(topic["result"]["topic"].get("note").is_none());
}

#[test]
fn set_labels_dry_run_reports_updated_topic_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "id:topic-payment",
            "--set-labels",
            "MVP,Payments",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["updated"]["id"], "topic-payment");
    assert_eq!(body["result"]["updated"]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["updated"]["changed_fields"][0], "labels");
    assert_eq!(body["result"]["updated"]["new_labels"][0], "MVP");
    assert_eq!(body["result"]["updated"]["new_labels"][1], "Payments");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "id:topic-payment",
            "--json",
        ])
        .output()
        .expect("get command runs after dry run");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(
        topic["result"]["topic"]["labels"]
            .as_array()
            .expect("labels is an array")
            .len(),
        0
    );
}

#[test]
fn set_append_note_dry_run_reports_updated_topic_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            "tests/fixtures/xmind/metadata.xmind",
            "--node",
            "id:topic-payment",
            "--append-note",
            " Extra context.",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["updated"]["id"], "topic-payment");
    assert_eq!(body["result"]["updated"]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["updated"]["changed_fields"][0], "note");
    assert_eq!(
        body["result"]["updated"]["new_note"],
        "Supports card payments and refund workflows. Extra context."
    );

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/metadata.xmind",
            "--node",
            "id:topic-payment",
            "--json",
        ])
        .output()
        .expect("get command runs after dry run");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(
        topic["result"]["topic"]["note"],
        "Supports card payments and refund workflows."
    );
}

#[test]
fn set_title_apply_writes_updated_topic() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-set.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--title",
            "Payments",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["new_path"], "/Q2/Payments");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "2"])
        .output()
        .expect("tree command runs after apply");
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(
        tree["result"]["root"]["children"][0]["children"][0]["title"],
        "Payments"
    );
    assert_eq!(
        tree["result"]["root"]["children"][0]["children"][0]["path"],
        "/Q2/Payments"
    );
}

#[test]
fn set_note_apply_writes_updated_topic_note() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-set-note.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--note",
            "Refund details",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["updated"]["changed_fields"][0], "note");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(topic["result"]["topic"]["note"], "Refund details");
}

#[test]
fn set_labels_apply_writes_updated_topic_labels() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-set-labels.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--set-labels",
            "MVP,Payments",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["new_labels"][0], "MVP");
    assert_eq!(body["result"]["updated"]["new_labels"][1], "Payments");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(topic["result"]["topic"]["labels"][0], "MVP");
    assert_eq!(topic["result"]["topic"]["labels"][1], "Payments");
}

#[test]
fn set_add_label_apply_appends_topic_label() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-add-label.xmind");
    fs::copy("tests/fixtures/xmind/metadata.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--add-label",
            "Urgent",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["new_labels"][0], "MVP");
    assert_eq!(body["result"]["updated"]["new_labels"][1], "Payments");
    assert_eq!(body["result"]["updated"]["new_labels"][2], "Urgent");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(topic["result"]["topic"]["labels"][0], "MVP");
    assert_eq!(topic["result"]["topic"]["labels"][1], "Payments");
    assert_eq!(topic["result"]["topic"]["labels"][2], "Urgent");
}

#[test]
fn set_remove_label_apply_removes_topic_label() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-remove-label.xmind");
    fs::copy("tests/fixtures/xmind/metadata.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--remove-label",
            "MVP",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["new_labels"][0], "Payments");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    let labels = topic["result"]["topic"]["labels"]
        .as_array()
        .expect("labels is an array");
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], "Payments");
}

#[test]
fn set_markers_apply_replaces_topic_markers() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-set-markers.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--set-markers",
            "priority-1,task-start",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["new_markers"][0], "priority-1");
    assert_eq!(body["result"]["updated"]["new_markers"][1], "task-start");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(topic["result"]["topic"]["markers"][0], "priority-1");
    assert_eq!(topic["result"]["topic"]["markers"][1], "task-start");
}

#[test]
fn set_add_marker_apply_appends_topic_marker() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-add-marker.xmind");
    fs::copy("tests/fixtures/xmind/metadata.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--add-marker",
            "flag-red",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["new_markers"][0], "priority-1");
    assert_eq!(body["result"]["updated"]["new_markers"][1], "task-start");
    assert_eq!(body["result"]["updated"]["new_markers"][2], "flag-red");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(topic["result"]["topic"]["markers"][2], "flag-red");
}

#[test]
fn set_remove_marker_apply_removes_topic_marker() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-remove-marker.xmind");
    fs::copy("tests/fixtures/xmind/metadata.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--remove-marker",
            "priority-1",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["new_markers"][0], "task-start");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    let markers = topic["result"]["topic"]["markers"]
        .as_array()
        .expect("markers is an array");
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0], "task-start");
}

#[test]
fn set_hyperlink_apply_writes_topic_hyperlink() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-set-hyperlink.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--hyperlink",
            "https://example.com/payments",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["applied"], true);
    assert_eq!(
        body["result"]["updated"]["new_hyperlink"],
        "https://example.com/payments"
    );

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(
        topic["result"]["topic"]["hyperlink"],
        "https://example.com/payments"
    );
}

#[test]
fn set_clear_repeated_apply_clears_topic_fields() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-set-clear.xmind");
    fs::copy("tests/fixtures/xmind/metadata.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
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
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["changed_fields"][0], "note");
    assert_eq!(body["result"]["updated"]["changed_fields"][1], "labels");
    assert_eq!(body["result"]["updated"]["changed_fields"][2], "markers");
    assert_eq!(body["result"]["updated"]["changed_fields"][3], "hyperlink");

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert!(topic["result"]["topic"].get("note").is_none());
    assert!(topic["result"]["topic"].get("hyperlink").is_none());
    assert_eq!(
        topic["result"]["topic"]["labels"]
            .as_array()
            .expect("labels is an array")
            .len(),
        0
    );
    assert_eq!(
        topic["result"]["topic"]["markers"]
            .as_array()
            .expect("markers is an array")
            .len(),
        0
    );
}

#[test]
fn set_clear_rejects_comma_separated_fields() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            "tests/fixtures/xmind/metadata.xmind",
            "--node",
            "id:topic-payment",
            "--clear",
            "labels,markers",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(2));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "invalid_usage");
    assert_eq!(
        body["error"]["suggested_fix"],
        "Pass one field per --clear flag; comma-separated clear fields are not supported."
    );
}

#[test]
fn set_append_note_apply_writes_appended_topic_note() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("apply-append-note.xmind");
    fs::copy("tests/fixtures/xmind/metadata.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &workbook_arg,
            "--node",
            "id:topic-payment",
            "--append-note",
            " Extra context.",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json set output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "set");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["updated"]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["updated"]["changed_fields"][0], "note");
    assert_eq!(
        body["result"]["updated"]["new_note"],
        "Supports card payments and refund workflows. Extra context."
    );

    let get_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["get", &workbook_arg, "--node", "id:topic-payment", "--json"])
        .output()
        .expect("get command runs after apply");
    let topic: Value = serde_json::from_slice(&get_output.stdout).expect("get stdout is JSON");
    assert_eq!(
        topic["result"]["topic"]["note"],
        "Supports card payments and refund workflows. Extra context."
    );
}
