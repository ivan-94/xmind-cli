#![cfg(target_os = "macos")]

use assert_cmd::Command;
use serde_json::Value;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[test]
fn tree_materializes_hidden_icloud_placeholder_before_reading_workbook() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    write_icloud_placeholder_for(&workbook);
    let tools = FakeCloudTools::new();
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = tools
        .command()
        .args(["tree", &workbook_arg, "--json", "--depth", "1"])
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
    assert_eq!(body["workbook"], workbook_arg);
    assert_eq!(body["result"]["root"]["title"], "Roadmap");
    assert!(workbook.exists(), "workbook should be materialized");
}

#[test]
fn direct_icloud_placeholder_argument_is_normalized_to_workbook_path() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    let placeholder = write_icloud_placeholder_for(&workbook);
    let tools = FakeCloudTools::new();
    let workbook_arg = workbook.to_string_lossy().into_owned();
    let placeholder_arg = placeholder.to_string_lossy().into_owned();

    let output = tools
        .command()
        .args(["tree", &placeholder_arg, "--json", "--depth", "1"])
        .output()
        .expect("tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["workbook"], workbook_arg);
    assert!(workbook.exists(), "workbook should be materialized");
}

#[test]
fn brctl_download_is_used_when_fileprovider_materialize_fails() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    write_icloud_placeholder_for(&workbook);
    let tools = FakeCloudTools::new();
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = tools
        .command()
        .env("XMIND_FAKE_FILEPROVIDER_FAIL", "1")
        .args(["tree", &workbook_arg, "--json", "--depth", "1"])
        .output()
        .expect("tree command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    let log = tools.log();
    assert!(
        log.contains("fileproviderctl materialize"),
        "fileproviderctl should be attempted first: {log}"
    );
    assert!(
        log.contains("brctl download"),
        "brctl should be attempted after fileproviderctl fails: {log}"
    );
}

#[test]
fn cloud_download_failure_returns_structured_retryable_error() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    write_icloud_placeholder_for(&workbook);
    let tools = FakeCloudTools::new();
    let workbook_arg = workbook.to_string_lossy().into_owned();

    let output = tools
        .command()
        .env("XMIND_FAKE_FILEPROVIDER_FAIL", "1")
        .env("XMIND_FAKE_BRCTL_FAIL", "1")
        .args(["tree", &workbook_arg, "--json", "--depth", "1"])
        .output()
        .expect("tree command runs");

    assert_eq!(output.status.code(), Some(12));
    assert!(
        output.stderr.is_empty(),
        "json runtime errors should be emitted on stdout: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], false);
    assert_eq!(body["command"], "tree");
    assert_eq!(body["workbook"], workbook_arg);
    assert_eq!(body["error"]["code"], "cloud_download_failed");
    assert_eq!(body["error"]["retryable"], true);
    assert_eq!(body["error"]["path"], workbook_arg);
    assert_eq!(body["error"]["exit_code"], 12);
    assert_eq!(body["error"]["details"]["logical_path"], workbook_arg);
    assert_eq!(
        body["error"]["details"]["attempts"][0]["tool"],
        "fileproviderctl"
    );
    assert_eq!(body["error"]["details"]["attempts"][1]["tool"], "brctl");
    assert!(
        !workbook.exists(),
        "failed materialization should not create the workbook"
    );
}

#[test]
fn add_apply_materializes_icloud_workbook_before_backup_and_write() {
    let temp_dir = tempfile::tempdir().expect("temp dir is created");
    let workbook = temp_dir.path().join("roadmap.xmind");
    write_icloud_placeholder_for(&workbook);
    let tools = FakeCloudTools::new();
    let workbook_arg = workbook.to_string_lossy().into_owned();
    let original_bytes =
        fs::read("tests/fixtures/xmind/minimal.xmind").expect("fixture is readable");

    let output = tools
        .command()
        .args([
            "add",
            &workbook_arg,
            "--parent",
            "path:/Q2",
            "--title",
            "Refund",
            "--apply",
            "--backup",
            "--json",
        ])
        .output()
        .expect("add command runs");

    assert_eq!(output.status.code(), Some(0));
    let body: Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(body["ok"], true);
    assert_eq!(body["workbook"], workbook_arg);
    assert_eq!(body["applied"], true);
    assert_eq!(body["result"]["created"]["path"], "/Q2/Refund");

    let backup_path = body["result"]["backup_path"]
        .as_str()
        .expect("backup path is returned");
    assert_eq!(
        fs::read(backup_path).expect("backup is readable"),
        original_bytes
    );

    let validate = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(["validate", &workbook_arg, "--json"])
        .output()
        .expect("validate command runs");
    assert_eq!(validate.status.code(), Some(0));
    let validate_body: Value = serde_json::from_slice(&validate.stdout).expect("stdout is JSON");
    assert_eq!(validate_body["result"]["valid"], true);
}

struct FakeCloudTools {
    _dir: tempfile::TempDir,
    path_env: std::ffi::OsString,
    log_path: PathBuf,
}

impl FakeCloudTools {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("fake tool dir is created");
        let log_path = dir.path().join("cloud-tools.log");
        write_tool(
            &dir.path().join("fileproviderctl"),
            r#"#!/bin/sh
set -eu
printf '%s %s\n' "fileproviderctl" "$*" >> "$XMIND_FAKE_CLOUD_LOG"
if [ "${XMIND_FAKE_FILEPROVIDER_FAIL:-0}" = "1" ]; then
  exit 42
fi
if [ "$1" != "materialize" ]; then
  exit 2
fi
cp "$XMIND_FAKE_WORKBOOK_SOURCE" "$2"
"#,
        );
        write_tool(
            &dir.path().join("brctl"),
            r#"#!/bin/sh
set -eu
printf '%s %s\n' "brctl" "$*" >> "$XMIND_FAKE_CLOUD_LOG"
if [ "${XMIND_FAKE_BRCTL_FAIL:-0}" = "1" ]; then
  exit 42
fi
if [ "$1" != "download" ]; then
  exit 2
fi
cp "$XMIND_FAKE_WORKBOOK_SOURCE" "$2"
"#,
        );

        let path_env = env::join_paths(
            std::iter::once(dir.path().to_path_buf())
                .chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())),
        )
        .expect("fake PATH is valid");

        Self {
            _dir: dir,
            path_env,
            log_path,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::cargo_bin("xmind").expect("xmind binary is built for CLI tests");
        command
            .env("PATH", &self.path_env)
            .env(
                "XMIND_FAKE_WORKBOOK_SOURCE",
                "tests/fixtures/xmind/minimal.xmind",
            )
            .env("XMIND_FAKE_CLOUD_LOG", &self.log_path);
        command
    }

    fn log(&self) -> String {
        fs::read_to_string(&self.log_path).unwrap_or_default()
    }
}

fn write_icloud_placeholder_for(workbook: &Path) -> PathBuf {
    let file_name = workbook
        .file_name()
        .and_then(|name| name.to_str())
        .expect("workbook file name is UTF-8");
    let placeholder = workbook.with_file_name(format!(".{file_name}.icloud"));
    fs::write(&placeholder, b"iCloud placeholder").expect("iCloud placeholder is written");
    placeholder
}

fn write_tool(path: &Path, body: &str) {
    fs::write(path, body).expect("fake cloud tool is written");
    let mut permissions = fs::metadata(path)
        .expect("fake cloud tool metadata is readable")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("fake cloud tool is executable");
}
