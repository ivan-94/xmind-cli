use std::fs;
#[cfg(unix)]
use std::process::Command;

#[test]
fn cargo_dist_release_contract_is_checked_in() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml is readable");
    let release_workflow = fs::read_to_string(".github/workflows/release.yml")
        .expect("cargo-dist release workflow should be checked in");
    let release_policy =
        fs::read_to_string("docs/technical/release-policy.md").expect("release policy is readable");
    let installation =
        fs::read_to_string("docs/installation.md").expect("installation doc is readable");
    let readme = fs::read_to_string("README.md").expect("README is readable");
    let changelog = fs::read_to_string("CHANGELOG.md").expect("changelog is readable");

    for expected in [
        r#"repository = "https://github.com/ivan-94/xmind-cli""#,
        "[profile.dist]",
        r#"inherits = "release""#,
        "[workspace.metadata.dist]",
        r#"cargo-dist-version = "0.31.0""#,
        r#"homepage = "https://github.com/ivan-94/xmind-cli""#,
        r#"allow-dirty = ["ci"]"#,
        r#"ci = ["github"]"#,
        r#"hosting = ["github"]"#,
        r#"installers = []"#,
        r#"checksum = "sha256""#,
        r#"create-release = true"#,
        r#"pr-run-mode = "plan""#,
        r#"unix-archive = ".tar.gz""#,
        r#"dist = true"#,
        r#""aarch64-apple-darwin""#,
        r#""aarch64-unknown-linux-gnu""#,
        r#""x86_64-apple-darwin""#,
        r#""x86_64-unknown-linux-gnu""#,
        r#""x86_64-pc-windows-msvc""#,
    ] {
        assert!(
            cargo_toml.contains(expected),
            "Cargo.toml should include `{expected}`"
        );
    }

    for expected in [
        "on:",
        "push:",
        "tags:",
        "v*",
        "pull_request:",
        "permissions:",
        "contents: write",
        "id-token: write",
        "dist plan",
        "dist build",
        "cargo-dist-installer.sh",
        "https://github.com/axodotdev/cargo-dist/releases/download/v0.31.0/cargo-dist-installer.sh",
        "CARGO_DIST_NO_MODIFY_PATH=1",
        "CARGO_DIST_PRINT_QUIET=1",
        r#""$dist_bin/dist" --version"#,
        "GITHUB_PATH",
        "cygpath -w",
        "aarch64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "macos-14",
        "ubuntu-24.04-arm",
        "macos-15-intel",
        "ubuntu-latest",
        "windows-latest",
        "./target/aarch64-apple-darwin/dist/xmind",
        "./target/aarch64-unknown-linux-gnu/dist/xmind",
        "./target/x86_64-apple-darwin/dist/xmind",
        "./target/x86_64-unknown-linux-gnu/dist/xmind",
        "./target/x86_64-pc-windows-msvc/dist/xmind.exe",
        "Smoke release binary",
        "${{ matrix.binary }} --version",
        "${{ matrix.binary }} tree tests/fixtures/xmind/minimal.xmind --json",
        "${{ matrix.binary }} validate tests/fixtures/xmind/minimal.xmind --json",
        "Generate aggregate SHA256SUMS",
        "find . -maxdepth 1 -type f ! -name 'SHA256SUMS' ! -name '*.sha256' -exec basename",
        "shasum -a 256",
        "cat SHA256SUMS",
        "Extract release notes",
        "bash .github/scripts/extract-release-notes.sh",
        "${{ github.ref_name }}",
        "target/release-notes.md",
        "GitHub Release",
        "Publish Homebrew formula",
        "HOMEBREW_TAP_TOKEN",
        "bash .github/scripts/update-homebrew-formula.sh",
    ] {
        assert!(
            release_workflow.contains(expected),
            "release workflow should include `{expected}`"
        );
    }

    let checksum_step = release_workflow
        .find("Generate aggregate SHA256SUMS")
        .expect("release workflow should generate aggregate SHA256SUMS");
    let release_step = release_workflow
        .find("Publish GitHub Release")
        .expect("release workflow should publish GitHub Release");
    assert!(
        checksum_step < release_step,
        "SHA256SUMS must be generated before GitHub Release publication"
    );
    assert!(
        release_workflow.contains("files: target/distrib/*"),
        "GitHub Release should upload SHA256SUMS with target/distrib artifacts"
    );
    assert!(
        release_workflow.contains("body_path: target/release-notes.md"),
        "GitHub Release should publish only the changelog section extracted for the release tag"
    );
    assert!(
        !release_workflow.contains("body_path: CHANGELOG.md"),
        "GitHub Release must not publish the whole changelog or Unreleased section"
    );

    assert!(
        !cargo_toml.contains(r#""homebrew""#) && !cargo_toml.contains("publish-jobs"),
        "cargo-dist config should keep Homebrew publication in the checked-in release workflow script"
    );

    for expected in [
        "cargo-dist",
        "GitHub Releases",
        "CHANGELOG.md",
        "SHA256",
        "per-artifact `.sha256` checksum files",
        "`github-release` job generates `SHA256SUMS`",
        "Supported Release Platforms",
        "macOS Apple Silicon",
        "macOS Intel",
        "Linux x86_64 GNU",
        "Windows x86_64 MSVC",
        "Unsupported Platforms",
        "Full E2E matrix remains separate",
        "issue #5",
        "issue #6",
    ] {
        assert!(
            release_policy.contains(expected),
            "release policy should document `{expected}`"
        );
    }

    for expected in [
        "cargo dist plan",
        "cargo dist build --artifacts=local --target aarch64-apple-darwin",
        "cargo dist build --artifacts=local --target aarch64-unknown-linux-gnu",
        "cargo dist build --artifacts=local --target x86_64-apple-darwin",
        "cargo dist build --artifacts=local --target x86_64-unknown-linux-gnu",
        "cargo dist build --artifacts=local --target x86_64-pc-windows-msvc",
        "xmind tree tests/fixtures/xmind/minimal.xmind --json",
        "xmind validate tests/fixtures/xmind/minimal.xmind --json",
        "per-artifact `.sha256` checksum files",
        "bash scripts/install.sh --dry-run --version v0.1.1",
        "cargo install --locked --git https://github.com/ivan-94/xmind-cli",
        "Homebrew",
        "brew install ivan-94/tap/xmind-cli",
        "install script",
    ] {
        assert!(
            installation.contains(expected),
            "installation doc should mention `{expected}`"
        );
    }

    for expected in [
        "macOS Apple Silicon",
        "macOS Intel",
        "Linux arm64 GNU",
        "Linux x86_64 GNU",
        "Windows x86_64 MSVC",
        "Linux musl/static builds",
        "macOS universal binaries",
        "Windows GNU",
    ] {
        assert!(
            readme.contains(expected),
            "README should make release platform support explicit with `{expected}`"
        );
    }

    assert!(
        changelog.contains("cargo-dist release workflow"),
        "changelog should mention cargo-dist release workflow"
    );
}

#[cfg(unix)]
#[test]
fn release_notes_extraction_uses_matching_version_section_only() {
    let temp = tempfile::tempdir().expect("temp dir is created");
    let changelog = temp.path().join("CHANGELOG.md");
    let output = temp.path().join("release-notes.md");
    fs::write(
        &changelog,
        r#"# Changelog

## Unreleased

### Added

- Draft work that must not be published.

## v0.1.0 - 2026-05-23

### Added

- First release.

## v0.0.9 - 2026-05-22

### Fixed

- Older release.
"#,
    )
    .expect("changelog is written");

    let output_status = Command::new("bash")
        .arg(".github/scripts/extract-release-notes.sh")
        .arg("v0.1.0")
        .arg(&changelog)
        .arg(&output)
        .output()
        .expect("release note extraction starts");

    assert!(
        output_status.status.success(),
        "release note extraction should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output_status.stdout),
        String::from_utf8_lossy(&output_status.stderr)
    );

    let body = fs::read_to_string(&output).expect("release notes are written");
    assert!(body.contains("## v0.1.0 - 2026-05-23"));
    assert!(body.contains("- First release."));
    assert!(!body.contains("Draft work that must not be published."));
    assert!(!body.contains("Older release."));
}

#[cfg(unix)]
#[test]
fn release_notes_extraction_fails_when_version_section_is_missing() {
    let temp = tempfile::tempdir().expect("temp dir is created");
    let changelog = temp.path().join("CHANGELOG.md");
    let output = temp.path().join("release-notes.md");
    fs::write(
        &changelog,
        r#"# Changelog

## Unreleased

### Added

- Draft work.
"#,
    )
    .expect("changelog is written");

    let output_status = Command::new("bash")
        .arg(".github/scripts/extract-release-notes.sh")
        .arg("v0.1.0")
        .arg(&changelog)
        .arg(&output)
        .output()
        .expect("release note extraction starts");

    assert!(
        !output_status.status.success(),
        "release note extraction should fail without a matching version section"
    );
    let stderr = String::from_utf8_lossy(&output_status.stderr);
    assert!(stderr.contains("missing a release notes section for v0.1.0"));
    assert!(
        !output.exists(),
        "missing release notes section must not leave a body file for publication"
    );
}

#[test]
fn homebrew_tap_path_is_documented_as_active_channel_with_formula_automation() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml is readable");
    let release_workflow = fs::read_to_string(".github/workflows/release.yml")
        .expect("cargo-dist release workflow should be checked in");
    let homebrew_script = fs::read_to_string(".github/scripts/update-homebrew-formula.sh")
        .expect("Homebrew formula update script should be checked in");
    let release_policy =
        fs::read_to_string("docs/technical/release-policy.md").expect("release policy is readable");
    let installation =
        fs::read_to_string("docs/installation.md").expect("installation doc is readable");
    let readme = fs::read_to_string("README.md").expect("README is readable");

    for expected in [
        "ivan-94/homebrew-tap",
        "active Homebrew",
        "brew install ivan-94/tap/xmind-cli",
        "HOMEBREW_TAP_TOKEN",
        "Homebrew formula checksums must come from the same published GitHub Release artifact checksums",
        "brew audit --strict --online",
        "brew test",
        "xmind --version",
    ] {
        assert!(
            release_policy.contains(expected),
            "release policy should document Homebrew tap path detail `{expected}`"
        );
    }

    for expected in [
        "Homebrew tap",
        "Available for tagged releases",
        "ivan-94/homebrew-tap",
        "brew install ivan-94/tap/xmind-cli",
    ] {
        assert!(
            readme.contains(expected),
            "README should describe Homebrew channel accurately with `{expected}`"
        );
    }

    for expected in [
        "Homebrew",
        "brew install ivan-94/tap/xmind-cli",
        "ivan-94/homebrew-tap",
        "HOMEBREW_TAP_TOKEN",
        "brew audit --strict --online",
        "brew test",
    ] {
        assert!(
            installation.contains(expected),
            "installation doc should document Homebrew channel condition `{expected}`"
        );
    }

    for expected in [
        "Formula/${FORMULA_NAME}.rb",
        "class ${class_name} < Formula",
        "bin.install \"xmind\"",
        "assert_match version.to_s",
        "git push origin HEAD:main",
        "artifact_for()",
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
    ] {
        assert!(
            homebrew_script.contains(expected),
            "Homebrew script should include `{expected}`"
        );
    }

    assert!(
        !cargo_toml.contains(r#""homebrew""#) && !cargo_toml.contains("publish-jobs"),
        "Homebrew publishing should remain in the repository-maintained workflow, not cargo-dist generated publish jobs"
    );
    assert!(
        release_workflow.contains("Publish Homebrew formula")
            && release_workflow.contains("HOMEBREW_TAP_TOKEN"),
        "release workflow should publish the tap formula after GitHub Release artifacts exist"
    );
}

#[cfg(unix)]
#[test]
fn homebrew_formula_update_script_writes_formula_to_tap_repo() {
    let release_dir = tempfile::tempdir().expect("release dir is created");
    let mut checksums = String::new();
    for (artifact, sha) in [
        (
            "xmind-cli-aarch64-apple-darwin.tar.gz",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        (
            "xmind-cli-x86_64-apple-darwin.tar.gz",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
        (
            "xmind-cli-aarch64-unknown-linux-gnu.tar.gz",
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ),
        (
            "xmind-cli-x86_64-unknown-linux-gnu.tar.gz",
            "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        ),
    ] {
        checksums.push_str(sha);
        checksums.push_str("  ");
        checksums.push_str(artifact);
        checksums.push('\n');
    }
    fs::write(release_dir.path().join("SHA256SUMS"), checksums).expect("SHA256SUMS is written");

    let seed = tempfile::tempdir().expect("seed tap dir is created");
    assert!(Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(seed.path())
        .status()
        .expect("git init starts")
        .success());
    fs::write(seed.path().join("README.md"), "# homebrew-tap\n").expect("README is written");
    assert!(Command::new("git")
        .args(["add", "README.md"])
        .current_dir(seed.path())
        .status()
        .expect("git add starts")
        .success());
    assert!(Command::new("git")
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "init"
        ])
        .current_dir(seed.path())
        .status()
        .expect("git commit starts")
        .success());

    let bare = tempfile::tempdir().expect("bare tap dir is created");
    assert!(Command::new("git")
        .args(["clone", "--bare"])
        .arg(seed.path())
        .arg(bare.path())
        .status()
        .expect("git clone --bare starts")
        .success());

    let output = Command::new("bash")
        .arg(".github/scripts/update-homebrew-formula.sh")
        .env("DIST_DIR", release_dir.path())
        .env("GITHUB_REF_NAME", "v0.1.0")
        .env("GITHUB_REPOSITORY", "ivan-94/xmind-cli")
        .env("HOMEBREW_TAP_TOKEN", "test-token")
        .env(
            "HOMEBREW_TAP_REMOTE_URL",
            format!("file://{}", bare.path().display()),
        )
        .env("GITHUB_ACTOR", "test")
        .env("GITHUB_ACTOR_ID", "1")
        .output()
        .expect("Homebrew formula update script starts");

    assert!(
        output.status.success(),
        "formula update should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let checkout = tempfile::tempdir().expect("tap checkout is created");
    assert!(Command::new("git")
        .args(["clone"])
        .arg(bare.path())
        .arg(checkout.path())
        .status()
        .expect("git clone starts")
        .success());
    let formula =
        fs::read_to_string(checkout.path().join("Formula/xmind-cli.rb")).expect("formula exists");

    for expected in [
        "class XmindCli < Formula",
        "url \"https://github.com/ivan-94/xmind-cli/releases/download/v0.1.0/xmind-cli-aarch64-apple-darwin.tar.gz\"",
        "sha256 \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"",
        "bin.install \"xmind\"",
        "assert_match version.to_s",
    ] {
        assert!(
            formula.contains(expected),
            "formula should include `{expected}`"
        );
    }
    assert!(
        !formula.contains("version \"0.1.0\""),
        "formula should let Homebrew infer the stable version from the release URL"
    );
}
