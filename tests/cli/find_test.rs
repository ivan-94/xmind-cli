use assert_cmd::Command;
use serde_json::Value;

#[test]
fn find_json_title_returns_exact_case_sensitive_matches() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--title",
            "Payment",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "find");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment");
    assert_eq!(body["result"]["matches"][0]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["matches"][0]["title"], "Payment");
    assert_eq!(body["result"]["matches"][0]["sheet"], "Roadmap");
    assert_eq!(body["result"]["matches"][0]["children_count"], 0);
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );

    let lower_case_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--title",
            "payment",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(lower_case_output.status.code(), Some(0));
    let lower_case_body: Value =
        serde_json::from_slice(&lower_case_output.stdout).expect("stdout is JSON");
    assert_eq!(
        lower_case_body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        0
    );
}

#[test]
fn find_json_title_contains_returns_case_sensitive_substring_matches() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--title-contains",
            "Pay",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "find");
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment");
    assert_eq!(body["result"]["matches"][0]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["matches"][0]["title"], "Payment");
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );

    let lower_case_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--title-contains",
            "pay",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(lower_case_output.status.code(), Some(0));
    let lower_case_body: Value =
        serde_json::from_slice(&lower_case_output.stdout).expect("stdout is JSON");
    assert_eq!(
        lower_case_body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        0
    );
}

#[test]
fn find_json_contains_searches_title_and_note_text() {
    let note_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/metadata.xmind",
            "--contains",
            "refund",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(note_output.status.code(), Some(0));
    assert!(
        note_output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&note_output.stderr)
    );

    let note_body: Value = serde_json::from_slice(&note_output.stdout).expect("stdout is JSON");
    assert_eq!(note_body["ok"], true);
    assert_eq!(note_body["command"], "find");
    assert_eq!(note_body["result"]["matches"][0]["id"], "topic-payment");
    assert_eq!(note_body["result"]["matches"][0]["path"], "/Q2/Payment");
    assert_eq!(note_body["result"]["matches"][0]["title"], "Payment");
    assert_eq!(
        note_body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );

    let title_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/metadata.xmind",
            "--contains",
            "Pay",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(title_output.status.code(), Some(0));
    let title_body: Value = serde_json::from_slice(&title_output.stdout).expect("stdout is JSON");
    assert_eq!(title_body["result"]["matches"][0]["id"], "topic-payment");
}

#[test]
fn find_json_query_title_equality_returns_matches() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--query",
            "title = \"Payment\"",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "find");
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment");
    assert_eq!(body["result"]["matches"][0]["path"], "/Q2/Payment");
    assert_eq!(body["result"]["matches"][0]["title"], "Payment");
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );
}

#[test]
fn find_json_query_title_inequality_returns_non_matching_titles() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--query",
            "title != \"Payment\"",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        2
    );
    assert_eq!(body["result"]["matches"][0]["id"], "topic-root");
    assert_eq!(body["result"]["matches"][1]["id"], "topic-q2");
}

#[test]
fn find_json_query_title_contains_returns_substring_matches() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--query",
            "title contains \"Pay\"",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment");
}

#[test]
fn find_json_query_title_starts_with_returns_prefix_matches() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--query",
            "title starts_with \"Pay\"",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment");
}

#[test]
fn find_json_query_title_ends_with_returns_suffix_matches() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--query",
            "title ends_with \"ment\"",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment");
}

#[test]
fn find_json_query_title_in_returns_list_matches() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--query",
            "title in [\"Payment\", \"Other\"]",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment");
}

#[test]
fn find_json_limit_truncates_matches_in_tree_order() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--title",
            "Payment",
            "--limit",
            "1",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "find");
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment-q1");
    assert_eq!(body["result"]["matches"][0]["path"], "/Q1/Payment");
}

#[test]
fn find_json_offset_skips_matches_before_limit() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/duplicate-titles.xmind",
            "--title",
            "Payment",
            "--offset",
            "1",
            "--limit",
            "1",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(
        body["result"]["matches"]
            .as_array()
            .expect("matches is an array")
            .len(),
        1
    );
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment-q2");
    assert_eq!(body["result"]["matches"][0]["path"], "/Q2/Payment");
}

#[test]
fn find_json_compact_format_limits_match_fields() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--title",
            "Payment",
            "--format",
            "compact-json",
            "--fields",
            "id,title",
            "--json",
        ])
        .output()
        .expect("find command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json find output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "find");
    assert_eq!(body["workbook"], "tests/fixtures/xmind/minimal.xmind");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["matches"][0]["id"], "topic-payment");
    assert_eq!(body["result"]["matches"][0]["title"], "Payment");
    assert!(body["result"]["matches"][0].get("path").is_none());
    assert!(body["result"]["matches"][0].get("sheet").is_none());
    assert!(body["result"]["matches"][0].get("children_count").is_none());
}
