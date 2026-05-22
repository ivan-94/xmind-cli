use assert_cmd::Command;
use serde_json::Value;

#[test]
fn tree_json_depth_one_reads_minimal_xmind_fixture() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--json",
            "--depth",
            "1",
        ])
        .output()
        .expect("tree command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "tree");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert_eq!(body["warnings"], Value::Array(vec![]));
    assert_eq!(body["result"]["sheet"], "Roadmap");
    assert_eq!(body["result"]["root"]["id"], "topic-root");
    assert_eq!(body["result"]["root"]["path"], "/");
    assert_eq!(body["result"]["root"]["title"], "Roadmap");
    assert_eq!(body["result"]["root"]["children"][0]["id"], "topic-q2");
    assert_eq!(body["result"]["root"]["children"][0]["path"], "/Q2");
    assert_eq!(body["result"]["root"]["children"][0]["children_count"], 1);
    assert!(body["result"]["root"]["children"][0]
        .get("children")
        .is_none());
}

#[test]
fn tree_json_include_assets_reads_topic_image_fixture() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/topic-image.xmind",
            "--json",
            "--include-assets",
            "--depth",
            "3",
        ])
        .output()
        .expect("tree command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    let payment = &body["result"]["root"]["children"][0]["children"][0];
    assert_eq!(payment["id"], "topic-payment");
    assert_eq!(payment["image"]["asset_id"], "xap:resources/payment.png");
    assert_eq!(payment["image"]["alt"], "Payment flow diagram");
    assert_eq!(payment["image"]["title"], "Payment flow");
}

#[test]
fn tree_json_can_select_sheet_by_index() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/multiple-sheets.xmind",
            "--sheet-index",
            "1",
            "--json",
            "--depth",
            "1",
        ])
        .output()
        .expect("tree command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["result"]["sheet"], "Backlog");
    assert_eq!(body["result"]["root"]["id"], "topic-backlog-root");
    assert_eq!(body["result"]["root"]["children"][0]["title"], "Ideas");
}

#[test]
fn tree_json_ambiguous_sheet_title_returns_candidates() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/duplicate-sheets.xmind",
            "--sheet",
            "Roadmap",
            "--json",
        ])
        .output()
        .expect("tree command runs");

    assert_eq!(output.status.code(), Some(6));
    assert!(
        output.stderr.is_empty(),
        "json tree errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "tree");
    assert_eq!(body["error"]["code"], "ambiguous_sheet");
    assert_eq!(body["error"]["details"]["sheet"], "Roadmap");
    assert_eq!(body["error"]["exit_code"], 6);
    assert_eq!(body["error"]["candidates"][0]["id"], "sheet-roadmap-a");
    assert_eq!(body["error"]["candidates"][0]["title"], "Roadmap");
    assert_eq!(body["error"]["candidates"][1]["id"], "sheet-roadmap-b");
    assert_eq!(body["error"]["candidates"][1]["title"], "Roadmap");
}

#[test]
fn tree_json_compact_format_limits_topic_fields() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--json",
            "--format",
            "compact-json",
            "--fields",
            "id,title,children",
            "--depth",
            "1",
        ])
        .output()
        .expect("tree command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "tree");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["result"]["sheet"], "Roadmap");
    assert_eq!(body["result"]["root"]["id"], "topic-root");
    assert_eq!(body["result"]["root"]["title"], "Roadmap");
    assert!(body["result"]["root"].get("path").is_none());
    assert!(body["result"]["root"].get("children_count").is_none());
    assert_eq!(body["result"]["root"]["children"][0]["id"], "topic-q2");
    assert_eq!(body["result"]["root"]["children"][0]["title"], "Q2");
    assert!(body["result"]["root"]["children"][0].get("path").is_none());
    assert!(body["result"]["root"]["children"][0]
        .get("children_count")
        .is_none());
}
