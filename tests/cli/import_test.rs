use assert_cmd::Command;
use serde_json::Value;
use std::fs;

#[test]
fn import_output_creates_new_workbook_from_yaml_tree() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let output_path = temp_dir.path().join("roadmap.xmind");
    let output_arg = output_path.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "import",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--output",
            &output_arg,
            "--apply",
            "--json",
        ])
        .output()
        .expect("import command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json import output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "import");
    assert_eq!(body["workbook"], output_arg);
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["output"], output_arg);
    assert_eq!(body["result"]["summary"]["added"], 9);
    assert!(
        output_path.exists(),
        "import should write the output workbook"
    );

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &output_arg, "--json", "--depth", "2"])
        .output()
        .expect("tree command runs on imported workbook");

    assert_eq!(tree_output.status.code(), Some(0));
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(tree["result"]["root"]["title"], "支付能力");
    assert_eq!(tree["result"]["root"]["children"][0]["title"], "收银台");
    assert_eq!(tree["result"]["root"]["children"][1]["title"], "退款");
}

#[test]
fn import_into_appends_tree_under_existing_parent() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "import",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--into",
            &workbook_arg,
            "--parent",
            "path:/Q2",
            "--apply",
            "--json",
        ])
        .output()
        .expect("import command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json import output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "import");
    assert_eq!(body["workbook"], workbook_arg);
    assert_eq!(body["result"]["parent"]["path"], "/Q2");
    assert_eq!(body["result"]["created_root"]["path"], "/Q2/支付能力");
    assert_eq!(body["result"]["summary"]["added"], 9);

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "3"])
        .output()
        .expect("tree command runs on imported workbook");

    assert_eq!(tree_output.status.code(), Some(0));
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    let q2_children = &tree["result"]["root"]["children"][0]["children"];
    assert_eq!(q2_children[0]["title"], "Payment");
    assert_eq!(q2_children[1]["title"], "支付能力");
    assert_eq!(q2_children[1]["children"][0]["title"], "收银台");
}
