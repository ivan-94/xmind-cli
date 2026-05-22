use assert_cmd::Command;
use serde_json::Value;

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
