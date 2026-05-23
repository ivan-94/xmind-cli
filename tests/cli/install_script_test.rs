#![cfg(unix)]

use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn install_script() -> PathBuf {
    PathBuf::from("scripts/install.sh")
}

#[test]
fn dry_run_previews_platform_artifact_without_writing_install_dir() {
    let install_dir = tempfile::tempdir().expect("install dir is created");

    let output = Command::new("bash")
        .arg(install_script())
        .args(["--dry-run", "--version", "v0.1.0"])
        .env("XMIND_INSTALL_OS", "Darwin")
        .env("XMIND_INSTALL_ARCH", "arm64")
        .env("XMIND_INSTALL_DIR", install_dir.path())
        .output()
        .expect("install script starts");

    assert!(
        output.status.success(),
        "dry-run should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DRY RUN"));
    assert!(stdout.contains("aarch64-apple-darwin"));
    assert!(stdout.contains("https://github.com/ivan-94/xmind-cli/releases/download/v0.1.0/"));
    assert!(stdout.contains("xmind-cli-v0.1.0-aarch64-apple-darwin.tar.gz"));
    assert!(
        fs::read_dir(install_dir.path())
            .expect("install dir can be listed")
            .next()
            .is_none(),
        "dry-run must not write files into the install dir"
    );
}

#[test]
fn installs_local_release_archive_after_checksum_verification() {
    let fixture = local_release_fixture("v0.1.0", "x86_64-unknown-linux-gnu", "ok");
    let install_dir = tempfile::tempdir().expect("install dir is created");

    let output = Command::new("bash")
        .arg(install_script())
        .args(["--version", "v0.1.0"])
        .env("XMIND_INSTALL_OS", "Linux")
        .env("XMIND_INSTALL_ARCH", "x86_64")
        .env("XMIND_INSTALL_DIR", install_dir.path())
        .env("XMIND_INSTALL_BASE_URL", fixture.release_dir.path())
        .output()
        .expect("install script starts");

    assert!(
        output.status.success(),
        "install should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let installed = install_dir.path().join("xmind");
    assert!(installed.exists(), "binary should be installed");
    assert_eq!(
        fs::read_to_string(&installed).expect("installed binary is readable"),
        "#!/bin/sh\necho xmind fixture\n"
    );
    assert_ne!(
        fs::metadata(&installed)
            .expect("installed metadata is readable")
            .permissions()
            .mode()
            & 0o111,
        0,
        "installed binary should be executable"
    );
}

#[test]
fn missing_checksum_file_fails_with_actionable_error_and_no_install() {
    let fixture = local_release_fixture("v0.1.0", "x86_64-unknown-linux-gnu", "missing");
    let install_dir = tempfile::tempdir().expect("install dir is created");

    let output = Command::new("bash")
        .arg(install_script())
        .args(["--version", "v0.1.0"])
        .env("XMIND_INSTALL_OS", "Linux")
        .env("XMIND_INSTALL_ARCH", "x86_64")
        .env("XMIND_INSTALL_DIR", install_dir.path())
        .env("XMIND_INSTALL_BASE_URL", fixture.release_dir.path())
        .output()
        .expect("install script starts");

    assert!(
        !output.status.success(),
        "install should fail when SHA256SUMS is missing"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to download"));
    assert!(stderr.contains("Refusing to install without checksums"));
    assert!(
        !install_dir.path().join("xmind").exists(),
        "missing checksum file must not install a binary"
    );
}

#[test]
fn checksum_mismatch_fails_with_actionable_error_and_no_install() {
    let fixture = local_release_fixture("v0.1.0", "x86_64-unknown-linux-gnu", "bad");
    let install_dir = tempfile::tempdir().expect("install dir is created");

    let output = Command::new("bash")
        .arg(install_script())
        .args(["--version", "v0.1.0"])
        .env("XMIND_INSTALL_OS", "Linux")
        .env("XMIND_INSTALL_ARCH", "x86_64")
        .env("XMIND_INSTALL_DIR", install_dir.path())
        .env("XMIND_INSTALL_BASE_URL", fixture.release_dir.path())
        .output()
        .expect("install script starts");

    assert!(
        !output.status.success(),
        "install should fail on checksum mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("checksum verification failed"));
    assert!(stderr.contains("delete the downloaded file"));
    assert!(
        !install_dir.path().join("xmind").exists(),
        "failed checksum must not install a binary"
    );
}

#[test]
fn unsupported_platform_fails_before_download() {
    let output = Command::new("bash")
        .arg(install_script())
        .arg("--dry-run")
        .env("XMIND_INSTALL_OS", "Linux")
        .env("XMIND_INSTALL_ARCH", "aarch64")
        .output()
        .expect("install script starts");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported platform"));
    assert!(stderr.contains("cargo install --locked --git"));
}

struct LocalReleaseFixture {
    release_dir: tempfile::TempDir,
}

fn local_release_fixture(version: &str, target: &str, checksum_mode: &str) -> LocalReleaseFixture {
    let release_dir = tempfile::tempdir().expect("release fixture dir is created");
    let archive_name = format!("xmind-cli-{version}-{target}.tar.gz");
    let archive_path = release_dir.path().join(&archive_name);
    let payload_dir = tempfile::tempdir().expect("payload dir is created");
    let bin_dir = payload_dir
        .path()
        .join(format!("xmind-cli-{version}-{target}"));
    fs::create_dir_all(&bin_dir).expect("payload bin dir is created");
    let binary = bin_dir.join("xmind");
    fs::write(&binary, "#!/bin/sh\necho xmind fixture\n").expect("fixture binary is written");
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755))
        .expect("fixture binary is executable");

    let status = Command::new("tar")
        .args(["-czf"])
        .arg(&archive_path)
        .arg("-C")
        .arg(payload_dir.path())
        .arg(".")
        .status()
        .expect("tar command starts");
    assert!(status.success(), "tar should create fixture archive");

    if checksum_mode != "missing" {
        let checksum = match checksum_mode {
            "ok" => sha256_file(&archive_path),
            "bad" => "0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            other => panic!("unknown checksum mode {other}"),
        };
        fs::write(
            release_dir.path().join("SHA256SUMS"),
            format!("{checksum}  {archive_name}\n"),
        )
        .expect("SHA256SUMS is written");
    }

    LocalReleaseFixture { release_dir }
}

fn sha256_file(path: &Path) -> String {
    let bytes = fs::read(path).expect("file is readable for checksum");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
