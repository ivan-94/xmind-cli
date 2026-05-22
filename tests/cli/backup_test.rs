use std::fs;

use assert_cmd::Command;
use serde_json::Value;

#[test]
fn backup_json_writes_to_custom_backup_dir() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let backup_dir = temp_dir.path().join("custom-backups");

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "backup",
            &workbook.to_string_lossy(),
            "--backup-dir",
            &backup_dir.to_string_lossy(),
            "--json",
        ])
        .output()
        .expect("backup command runs");

    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stderr.is_empty(),
        "json backup output should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "backup");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], false);

    let backup_path = body["result"]["backup_path"]
        .as_str()
        .expect("backup path is returned");
    assert!(backup_path.starts_with(backup_dir.to_str().expect("backup dir is UTF-8")));
    assert!(backup_path.ends_with(".xmind"));
    assert_eq!(
        fs::read(backup_path).expect("backup is readable"),
        fs::read(&workbook).expect("source is readable")
    );
}
