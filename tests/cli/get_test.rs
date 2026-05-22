use assert_cmd::Command;
use serde_json::Value;

#[test]
fn get_json_path_selector_returns_topic_with_requested_depth() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "path:/Q2",
            "--depth",
            "1",
            "--json",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json get output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "get");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["result"]["topic"]["id"], "topic-q2");
    assert_eq!(body["result"]["topic"]["path"], "/Q2");
    assert_eq!(body["result"]["topic"]["title"], "Q2");
    assert_eq!(
        body["result"]["topic"]["children"][0]["id"],
        "topic-payment"
    );
    assert_eq!(
        body["result"]["topic"]["children"][0]["path"],
        "/Q2/Payment"
    );
    assert_eq!(body["result"]["topic"]["children"][0]["title"], "Payment");
}

#[test]
fn get_json_query_selector_returns_unique_match() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "query:title = \"Payment\"",
            "--json",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json get output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "get");
    assert_eq!(body["result"]["topic"]["id"], "topic-payment");
    assert_eq!(body["result"]["topic"]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["topic"]["title"], "Payment");
}

#[test]
fn get_json_returns_topic_labels() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/metadata.xmind",
            "--node",
            "id:topic-payment",
            "--json",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json get output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "get");
    assert_eq!(body["result"]["topic"]["id"], "topic-payment");
    assert_eq!(body["result"]["topic"]["labels"][0], "MVP");
    assert_eq!(body["result"]["topic"]["labels"][1], "Payments");
}

#[test]
fn get_json_missing_selector_returns_not_found_diagnostic() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "path:/Missing",
            "--json",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(5));
    assert!(
        output.stderr.is_empty(),
        "json get errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "get");
    assert_eq!(body["error"]["code"], "not_found");
    assert_eq!(body["error"]["selector"], "path:/Missing");
    assert_eq!(body["error"]["exit_code"], 5);
}

#[test]
fn get_json_ambiguous_title_selector_returns_candidates() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--node",
            "title:Payment",
            "--json",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(6));
    assert!(
        output.stderr.is_empty(),
        "json get errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "get");
    assert_eq!(body["error"]["code"], "ambiguous_selector");
    assert_eq!(body["error"]["selector"], "title:Payment");
    assert_eq!(body["error"]["exit_code"], 6);
    assert_eq!(body["error"]["candidates"][0]["id"], "topic-payment-q1");
    assert_eq!(body["error"]["candidates"][0]["path"], "/Q1/Payment");
    assert_eq!(body["error"]["candidates"][0]["title"], "Payment");
    assert_eq!(body["error"]["candidates"][0]["sheet"], "Roadmap");
    assert_eq!(body["error"]["candidates"][1]["id"], "topic-payment-q2");
    assert_eq!(body["error"]["candidates"][1]["path"], "/Q2/Payment");
}

#[test]
fn get_json_can_select_sheet_by_title() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/multiple-sheets.xmind",
            "--sheet",
            "Backlog",
            "--node",
            "root",
            "--json",
            "--depth",
            "1",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["result"]["topic"]["id"], "topic-backlog-root");
    assert_eq!(body["result"]["topic"]["title"], "Backlog");
    assert_eq!(body["result"]["topic"]["children"][0]["title"], "Ideas");
}

#[test]
fn get_json_can_select_sheet_by_id() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/multiple-sheets.xmind",
            "--sheet-id",
            "sheet-backlog",
            "--node",
            "root",
            "--json",
            "--depth",
            "1",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["result"]["topic"]["id"], "topic-backlog-root");
    assert_eq!(body["result"]["topic"]["title"], "Backlog");
    assert_eq!(body["result"]["topic"]["children"][0]["title"], "Ideas");
}

#[test]
fn get_json_missing_sheet_returns_sheet_not_found() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/multiple-sheets.xmind",
            "--sheet",
            "Missing",
            "--node",
            "root",
            "--json",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(5));
    assert!(
        output.stderr.is_empty(),
        "json get errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "get");
    assert_eq!(body["error"]["code"], "sheet_not_found");
    assert_eq!(body["error"]["details"]["sheet"], "Missing");
    assert_eq!(body["error"]["exit_code"], 5);
}

#[test]
fn get_json_compact_format_limits_topic_fields() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "path:/Q2",
            "--depth",
            "1",
            "--format",
            "compact-json",
            "--fields",
            "id,title,children",
            "--json",
        ])
        .output()
        .expect("get command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json get output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "get");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["result"]["topic"]["id"], "topic-q2");
    assert_eq!(body["result"]["topic"]["title"], "Q2");
    assert!(body["result"]["topic"].get("path").is_none());
    assert!(body["result"]["topic"].get("children_count").is_none());
    assert_eq!(
        body["result"]["topic"]["children"][0]["id"],
        "topic-payment"
    );
    assert_eq!(body["result"]["topic"]["children"][0]["title"], "Payment");
    assert!(body["result"]["topic"]["children"][0].get("path").is_none());
    assert!(body["result"]["topic"]["children"][0]
        .get("children_count")
        .is_none());
}
