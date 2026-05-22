use std::fs;

use assert_cmd::Command;
use serde_json::Value;

#[test]
fn restore_dry_run_reports_latest_backup_without_writing() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let original = fs::read(&workbook).expect("workbook is readable");

    let backup_dir = temp_dir.path().join(".xmind-backups");
    fs::create_dir_all(&backup_dir).expect("backup dir is created");
    let backup = backup_dir.join("roadmap.2.xmind");
    fs::write(&backup, &original).expect("backup is written");
    fs::write(&workbook, b"corrupt current workbook").expect("workbook is overwritten");

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args([
            "restore",
            &workbook.to_string_lossy(),
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("restore command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "restore");
    assert_eq!(body["dry_run"], true);
    assert_eq!(body["applied"], false);
    assert_eq!(
        body["result"]["restored_from"].as_str(),
        Some(backup.to_string_lossy().as_ref())
    );
    assert_eq!(
        body["result"]["output"].as_str(),
        Some(workbook.to_string_lossy().as_ref())
    );
    assert_eq!(
        fs::read(&workbook).expect("workbook remains readable"),
        b"corrupt current workbook"
    );
}

#[test]
fn restore_apply_replaces_workbook_from_latest_backup() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    fs::copy("tests/fixtures/xmind/minimal.xmind", &workbook).expect("fixture is copied");
    let original = fs::read(&workbook).expect("workbook is readable");

    let backup_dir = temp_dir.path().join(".xmind-backups");
    fs::create_dir_all(&backup_dir).expect("backup dir is created");
    let backup = backup_dir.join("roadmap.2.xmind");
    fs::write(&backup, &original).expect("backup is written");
    fs::write(&workbook, b"corrupt current workbook").expect("workbook is overwritten");

    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["restore", &workbook.to_string_lossy(), "--apply", "--json"])
        .output()
        .expect("restore command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["command"], "restore");
    assert_eq!(body["dry_run"], false);
    assert_eq!(body["applied"], true);
    assert_eq!(
        body["result"]["restored_from"].as_str(),
        Some(backup.to_string_lossy().as_ref())
    );
    assert_eq!(fs::read(&workbook).expect("workbook is readable"), original);
}
