use assert_cmd::Command;
use serde_json::Value;

#[test]
fn add_tree_yaml_input_dry_run_reports_tree_diff_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json add-tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "add-tree");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["will_change"], true);
    assert_eq!(body["result"]["parent"]["path"], "/Q2");
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["summary"]["added"], 9);
    assert_eq!(body["result"]["diff"][0]["event"], "added");
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/支付能力");
    assert_eq!(
        body["result"]["diff"][3]["path"],
        "/Q2/支付能力/收银台/优惠券抵扣"
    );
    assert_eq!(body["result"]["diff"][8]["path"], "/Q2/支付能力/对账");

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
fn add_tree_json_input_dry_run_reports_tree_diff_without_writing() {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.json",
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("add-tree command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json add-tree output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "add-tree");
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["summary"]["added"], 8);
    assert_eq!(body["result"]["diff"][0]["path"], "/Q2/支付能力");
    assert_eq!(
        body["result"]["diff"][3]["path"],
        "/Q2/支付能力/收银台/优惠券抵扣"
    );
    assert_eq!(
        body["result"]["diff"][7]["path"],
        "/Q2/支付能力/退款/部分退款"
    );

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
