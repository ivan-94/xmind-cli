use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use zip::write::FileOptions;

pub const MINIMAL_FIXTURE: &str = "tests/fixtures/xmind/minimal.xmind";
pub const DUPLICATE_TITLES_FIXTURE: &str = "tests/fixtures/xmind/duplicate-titles.xmind";
pub const DUPLICATE_SHEETS_FIXTURE: &str = "tests/fixtures/xmind/duplicate-sheets.xmind";
pub const MALFORMED_FIXTURE: &str = "tests/fixtures/xmind/malformed.xmind";
pub const METADATA_FIXTURE: &str = "tests/fixtures/xmind/metadata.xmind";
pub const MULTIPLE_SHEETS_FIXTURE: &str = "tests/fixtures/xmind/multiple-sheets.xmind";
pub const TOPIC_IMAGE_FIXTURE: &str = "tests/fixtures/xmind/topic-image.xmind";

pub struct FixtureCopy {
    _temp_dir: TempDir,
    path: PathBuf,
    source: &'static str,
    source_bytes: Vec<u8>,
}

impl FixtureCopy {
    pub fn path_arg(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    pub fn assert_source_unchanged(&self) {
        let current = fs::read(self.source).expect("source fixture remains readable");
        assert_eq!(
            current, self.source_bytes,
            "mutating E2E commands must not modify committed fixture {}",
            self.source
        );
    }
}

pub fn copy_fixture(source: &'static str, file_name: &str) -> FixtureCopy {
    let temp_dir = tempfile::tempdir().expect("temp dir is created for E2E fixture copy");
    let path = temp_dir.path().join(file_name);
    fs::copy(source, &path).expect("fixture is copied before mutation");
    let source_bytes = fs::read(source).expect("source fixture is readable");

    FixtureCopy {
        _temp_dir: temp_dir,
        path,
        source,
        source_bytes,
    }
}

pub fn temp_file(dir: &Path, name: &str, content: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, content).expect("E2E temporary input is written");
    path.to_string_lossy().into_owned()
}

pub fn run_json(args: &[&str]) -> Value {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for E2E tests")
        .args(args)
        .output()
        .expect("xmind command runs");

    assert_eq!(
        output.status.code(),
        Some(0),
        "command should succeed: xmind {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "JSON commands should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("stdout is a JSON envelope")
}

pub fn run_json_error(args: &[&str], expected_exit: i32) -> Value {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for E2E tests")
        .args(args)
        .output()
        .expect("xmind command runs");

    assert_eq!(
        output.status.code(),
        Some(expected_exit),
        "command should fail with expected exit: xmind {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "JSON errors should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("error stdout is a JSON envelope")
}

pub fn run_success(args: &[&str]) {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for E2E tests")
        .args(args)
        .output()
        .expect("xmind command runs");

    assert_eq!(
        output.status.code(),
        Some(0),
        "command should succeed: xmind {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "successful E2E commands should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn run_human(args: &[&str]) -> String {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for E2E tests")
        .args(args)
        .output()
        .expect("xmind command runs");

    assert_eq!(
        output.status.code(),
        Some(0),
        "human command should succeed: xmind {}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "human commands in the PR subset should not emit stderr diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout is UTF-8")
}

pub fn run_human_error(args: &[&str], expected_exit: i32) -> String {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for E2E tests")
        .args(args)
        .output()
        .expect("xmind command runs");

    assert_eq!(
        output.status.code(),
        Some(expected_exit),
        "human command should fail with expected exit: xmind {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "human errors should not emit stdout diagnostics: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    String::from_utf8(output.stderr).expect("stderr is UTF-8")
}

pub fn assert_success_envelope(body: &Value, command: &str, workbook: Option<&str>) {
    assert_eq!(body["ok"], true, "{command} should return ok=true");
    assert_eq!(body["command"], command);
    assert!(body["dry_run"].is_boolean());
    assert!(body["applied"].is_boolean());
    assert!(
        body.get("result").is_some(),
        "{command} should return result"
    );

    if let Some(workbook) = workbook {
        assert_eq!(body["workbook"], workbook);
    }
}

pub fn validate_workbook(workbook: &str) {
    let body = run_json(&["validate", workbook, "--json"]);
    assert_success_envelope(&body, "validate", Some(workbook));
    assert_eq!(body["result"]["valid"], true);
}

pub fn write_unsupported_xmind_variant(path: &Path) {
    let file = File::create(path).expect("unsupported workbook fixture is created");
    let mut writer = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    writer
        .start_file("content.xml", options)
        .expect("content.xml entry starts");
    writer
        .write_all(b"<xmap-content></xmap-content>")
        .expect("content.xml is written");
    writer
        .finish()
        .expect("unsupported workbook zip is finished");
}
