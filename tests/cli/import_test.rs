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

#[test]
fn import_into_apply_with_backup_copies_original_before_replacing_workbook() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let original_bytes = fs::read(&workbook).expect("original workbook is readable");
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
            "--backup",
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
    let backup_path = body["result"]["backup_path"]
        .as_str()
        .expect("import --into --backup returns result.backup_path");
    assert!(backup_path.contains(".xmind-backups"));
    assert_eq!(
        fs::read(backup_path).expect("backup is readable"),
        original_bytes
    );

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "3"])
        .output()
        .expect("tree command runs on imported workbook");

    assert_eq!(tree_output.status.code(), Some(0));
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    let q2_children = &tree["result"]["root"]["children"][0]["children"];
    assert_eq!(q2_children[1]["title"], "支付能力");
}

#[test]
fn import_into_dry_run_with_backup_neither_writes_workbook_nor_backup() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let original_bytes = fs::read(&workbook).expect("original workbook is readable");
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
            "--dry-run",
            "--backup",
            "--json",
        ])
        .output()
        .expect("import command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert!(body["result"]["backup_path"].is_null());
    assert_eq!(
        fs::read(&workbook).expect("workbook remains readable"),
        original_bytes
    );
    assert!(
        !temp_dir.path().join(".xmind-backups").exists(),
        "dry-run import must not create a backup directory"
    );
}

#[test]
fn import_into_invalid_input_leaves_workbook_and_backup_directory_untouched() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let original_bytes = fs::read(&workbook).expect("original workbook is readable");
    let invalid_input = temp_dir.path().join("invalid.yaml");
    fs::write(&invalid_input, "title: ''\n").expect("invalid import input is written");
    let workbook_arg = workbook.to_string_lossy().into_owned();
    let input_arg = invalid_input.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "import",
            "--input",
            &input_arg,
            "--into",
            &workbook_arg,
            "--parent",
            "path:/Q2",
            "--apply",
            "--backup",
            "--json",
        ])
        .output()
        .expect("import command runs");

    assert_eq!(output.status.code(), Some(7));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["error"]["code"], "invalid_tree_input");
    assert_eq!(
        fs::read(&workbook).expect("workbook remains readable"),
        original_bytes
    );
    assert!(
        !temp_dir.path().join(".xmind-backups").exists(),
        "preflight failure must not create a backup"
    );
}

#[test]
fn import_output_rejects_backup_without_writing_target() {
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
            "--backup",
            "--json",
        ])
        .output()
        .expect("import command runs");

    assert_eq!(output.status.code(), Some(2));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["error"]["code"], "invalid_usage");
    assert!(
        !output_path.exists(),
        "rejected import output must not write the target workbook"
    );
}

#[test]
fn import_output_overwrite_replaces_existing_workbook() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let output_path = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &output_path).expect("fixture is copied");
    let output_arg = output_path.to_string_lossy().into_owned();

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "import",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--output",
            &output_arg,
            "--overwrite",
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
    assert_eq!(body["result"]["output"], output_arg);
    assert_eq!(body["result"]["summary"]["added"], 9);

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &output_arg, "--json", "--depth", "2"])
        .output()
        .expect("tree command runs on overwritten workbook");

    assert_eq!(tree_output.status.code(), Some(0));
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(tree["result"]["root"]["title"], "支付能力");
    assert_eq!(tree["result"]["root"]["children"][0]["title"], "收银台");
}

#[test]
fn import_output_without_overwrite_rejects_existing_workbook() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let output_path = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &output_path).expect("fixture is copied");
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

    assert_eq!(output.status.code(), Some(10));
    assert!(
        output.stderr.is_empty(),
        "json import errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"]["code"], "write_failed");

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &output_arg, "--json", "--depth", "1"])
        .output()
        .expect("tree command runs on preserved workbook");

    assert_eq!(tree_output.status.code(), Some(0));
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(tree["result"]["root"]["title"], "Roadmap");
}

#[test]
fn import_output_dry_run_reports_plan_without_creating_workbook() {
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
            "--dry-run",
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
    assert!(
        !output_path.exists(),
        "import --output --dry-run must not create the output workbook"
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "import");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(body["result"]["output"], output_arg);
    assert_eq!(body["result"]["summary"]["added"], 9);
}

#[test]
fn import_output_dry_run_reports_creation_diff_from_empty_workbook() {
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
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("import command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["result"]["diff"][0]["event"], "added");
    assert_eq!(body["result"]["diff"][0]["path"], "/");
    assert_eq!(body["result"]["diff"][1]["path"], "/收银台");
    assert_eq!(body["result"]["diff"][8]["path"], "/对账");
}

#[test]
fn export_markdown_then_import_output_round_trips_topic_outline() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let source_workbook_path = temp_dir.path().join("source.xmind");
    let markdown_path = temp_dir.path().join("roadmap.md");
    let workbook_path = temp_dir.path().join("roundtrip.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &source_workbook_path)
        .expect("source workbook is copied");
    let source_workbook_arg = source_workbook_path.to_string_lossy().into_owned();
    let markdown_arg = markdown_path.to_string_lossy().into_owned();
    let workbook_arg = workbook_path.to_string_lossy().into_owned();

    let set_note_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "set",
            &source_workbook_arg,
            "--node",
            "path:/Q2/Payment",
            "--note",
            "Supports card payments and refund workflows.",
            "--apply",
            "--json",
        ])
        .output()
        .expect("set note command runs");

    assert_eq!(set_note_output.status.code(), Some(0));
    assert!(
        set_note_output.stderr.is_empty(),
        "set note should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&set_note_output.stderr)
    );

    let export_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "export",
            &source_workbook_arg,
            "--format",
            "markdown",
            "--output",
            &markdown_arg,
        ])
        .output()
        .expect("export command runs");

    assert_eq!(export_output.status.code(), Some(0));
    assert!(
        export_output.stderr.is_empty(),
        "export should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&export_output.stderr)
    );

    let import_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "import",
            "--input",
            &markdown_arg,
            "--output",
            &workbook_arg,
            "--apply",
            "--json",
        ])
        .output()
        .expect("import command runs");

    assert_eq!(import_output.status.code(), Some(0));
    assert!(
        import_output.stderr.is_empty(),
        "import should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&import_output.stderr)
    );

    let tree_output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["tree", &workbook_arg, "--json", "--depth", "3"])
        .output()
        .expect("tree command runs on round-tripped workbook");

    assert_eq!(tree_output.status.code(), Some(0));
    let tree: Value = serde_json::from_slice(&tree_output.stdout).expect("tree stdout is JSON");
    assert_eq!(tree["result"]["root"]["title"], "Roadmap");
    assert_eq!(tree["result"]["root"]["children"][0]["title"], "Q2");
    assert_eq!(
        tree["result"]["root"]["children"][0]["children"][0]["title"],
        "Payment"
    );
    assert_eq!(
        tree["result"]["root"]["children"][0]["children"][0]["note"],
        "Supports card payments and refund workflows."
    );
}
