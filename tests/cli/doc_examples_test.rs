use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct MarkdownFence {
    path: PathBuf,
    line: usize,
    info: String,
    body: String,
}

#[test]
fn documented_bash_e2e_examples_are_extracted_and_run() {
    let examples = bash_e2e_examples(&[
        "docs/examples",
        "docs/guides",
        "docs/reference",
        "docs/technical/e2e-test-plan.md",
    ]);

    assert!(
        examples.iter().any(
            |example| example.path == Path::new("docs/examples/README.md")
                && example
                    .body
                    .contains("xmind tree tests/fixtures/xmind/minimal.xmind --json")
        ),
        "docs/examples/README.md should include a read-only bash e2e example"
    );
    assert!(
        examples.iter().any(
            |example| example.path == Path::new("docs/examples/README.md")
                && example
                    .body
                    .contains("cp tests/fixtures/xmind/minimal.xmind")
                && example.body.contains("--apply")
                && example.body.contains("xmind validate")
        ),
        "mutating bash e2e examples should copy a fixture into a temporary file before --apply"
    );

    for example in examples {
        run_bash_e2e(&example);
    }
}

#[test]
fn ordinary_bash_examples_remain_illustrative() {
    let readme = std::fs::read_to_string("docs/examples/README.md")
        .expect("docs examples README is readable");

    assert!(
        extract_markdown_fences(Path::new("docs/examples/README.md"), &readme)
            .iter()
            .any(|fence| fence.info == "bash"
                && fence
                    .body
                    .contains("xmind inspect tests/fixtures/xmind/minimal.xmind --json")),
        "ordinary bash examples should stay available as illustrative documentation"
    );
    assert!(
        bash_e2e_examples(&["docs/examples/README.md"])
            .iter()
            .all(|example| !example
                .body
                .contains("xmind inspect tests/fixtures/xmind/minimal.xmind --json")),
        "ordinary bash blocks must not be selected by the docs-example runner"
    );
}

#[test]
fn public_readmes_are_source_manifest_free_and_cross_linked() {
    let english = std::fs::read_to_string("README.md").expect("English README is readable");
    let chinese = std::fs::read_to_string("README.zh-CN.md").expect("Chinese README is readable");

    for (path, content) in [("README.md", &english), ("README.zh-CN.md", &chinese)] {
        assert!(
            !content.contains("Source Manifest"),
            "{path} is a public entrypoint and must not contain a Source Manifest section"
        );
        assert!(
            content.contains("0.1.0") && content.contains("Unreleased"),
            "{path} should describe the current early-release posture"
        );
    }

    assert!(
        english.contains("[中文](README.zh-CN.md)"),
        "English README should link to the Chinese README near the top"
    );
    assert!(
        chinese.contains("[English](README.md)"),
        "Chinese README should link to the English README near the top"
    );
}

fn bash_e2e_examples(paths: &[&str]) -> Vec<MarkdownFence> {
    let mut examples = Vec::new();
    for path in markdown_files(paths) {
        let content = fs::read_to_string(&path).expect("Markdown documentation is readable");
        examples.extend(
            extract_markdown_fences(&path, &content)
                .into_iter()
                .filter(|fence| fence.info == "bash e2e"),
        );
    }
    examples
}

fn markdown_files(paths: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        let path = Path::new(path);
        if path.is_dir() {
            for entry in fs::read_dir(path).expect("documentation directory is readable") {
                let entry = entry.expect("documentation entry is readable");
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    files.extend(markdown_files(&[entry_path
                        .to_str()
                        .expect("path is UTF-8")]));
                } else if entry_path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("md")
                {
                    files.push(entry_path);
                }
            }
        } else {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files
}

fn extract_markdown_fences(path: &Path, content: &str) -> Vec<MarkdownFence> {
    let mut fences = Vec::new();
    let mut active: Option<(usize, String, Vec<&str>)> = None;

    for (index, line) in content.lines().enumerate() {
        if let Some(rest) = line.strip_prefix("```") {
            if let Some((line, info, body)) = active.take() {
                fences.push(MarkdownFence {
                    path: path.to_path_buf(),
                    line,
                    info,
                    body: body.join("\n"),
                });
            } else {
                active = Some((index + 1, rest.trim().to_owned(), Vec::new()));
            }
        } else if let Some((_, _, body)) = active.as_mut() {
            body.push(line);
        }
    }

    fences
}

fn run_bash_e2e(example: &MarkdownFence) {
    let shim_dir = tempfile::tempdir().expect("PATH shim directory is created");
    let xmind_bin = assert_cmd::cargo::cargo_bin("xmind");
    let shim_path = shim_dir.path().join("xmind");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&xmind_bin, &shim_path).expect("xmind binary is linked into PATH");
    #[cfg(windows)]
    std::fs::copy(&xmind_bin, shim_path.with_extension("exe"))
        .expect("xmind binary is copied into PATH");

    let mut path_entries = vec![shim_dir.path().to_path_buf()];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(path_entries).expect("PATH entries can be joined");

    let output = std::process::Command::new("bash")
        .args(["-e", "-u", "-o", "pipefail", "-c", &example.body])
        .env("PATH", path)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "{}:{} failed to start bash e2e example: {error}",
                example.path.display(),
                example.line
            )
        });

    assert!(
        output.status.success(),
        "{}:{} bash e2e example failed\nscript:\n{}\nstdout:\n{}\nstderr:\n{}",
        example.path.display(),
        example.line,
        example.body,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn docs_do_not_advertise_removed_cli_surface() {
    let docs = [
        "docs/concepts",
        "docs/design",
        "docs/guides",
        "docs/product",
        "docs/reference",
        "docs/technical",
    ];
    let banned_literals = [
        "--validate-after",
        "--if-exists",
        "--match-by",
        "--include-notes",
        "xmind assets ",
        "xmind asset-export ",
    ];

    for doc_root in docs {
        for entry in std::fs::read_dir(doc_root).expect("documentation directory is readable") {
            let entry = entry.expect("documentation entry is readable");
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(&path).expect("documentation file is readable");
            for banned in banned_literals {
                assert!(
                    !content.contains(banned),
                    "{} still advertises removed CLI surface `{}`",
                    path.display(),
                    banned
                );
            }
            for line in content.lines() {
                assert!(
                    !(line.contains("xmind tree ") && line.contains(" --node ")),
                    "{} still advertises removed `xmind tree --node` surface: {}",
                    path.display(),
                    line
                );
                assert!(
                    !(line.contains("xmind restore ") && line.contains(" --output ")),
                    "{} still advertises removed `xmind restore --output` surface: {}",
                    path.display(),
                    line
                );
            }
        }
    }
}

#[test]
fn schema_docs_track_serializable_contracts() {
    let command_output = std::fs::read_to_string("docs/schemas/command-output.schema.md")
        .expect("schema is readable");
    assert!(
        !command_output.contains("\"validation\""),
        "command output schema should not document removed validation result payloads"
    );

    let error_schema =
        std::fs::read_to_string("docs/schemas/error.schema.md").expect("schema is readable");
    assert!(
        error_schema.contains("`operation`"),
        "error schema should document CliErrorBody.operation"
    );

    let patch_schema =
        std::fs::read_to_string("docs/schemas/patch.schema.md").expect("schema is readable");
    for field in ["`title`", "`by`", "`order`", "`recursive`", "`add_labels`"] {
        assert!(
            patch_schema.contains(field),
            "patch schema should document PatchOpDto field {field}"
        );
    }
    assert!(
        !patch_schema.contains("`if_exists`"),
        "patch schema should not document removed PatchOpDto.if_exists"
    );

    let topic_tree_schema =
        std::fs::read_to_string("docs/schemas/topic-tree.schema.md").expect("schema is readable");
    assert!(
        topic_tree_schema.contains("path: string?"),
        "topic tree schema should document TopicTreeInputDto.path"
    );
    assert!(
        !topic_tree_schema.contains("hyperlink"),
        "topic tree schema should not document unsupported TopicTreeInputDto.hyperlink"
    );
}

#[test]
fn technical_docs_track_implemented_modules() {
    let technical_docs = [
        "docs/technical/README.md",
        "docs/technical/architecture.md",
        "docs/technical/command-runtime.md",
        "docs/technical/crate-layout.md",
        "docs/technical/data-model.md",
        "docs/technical/quality-gates.md",
        "docs/technical/tech-stack.md",
        "docs/technical/testing-strategy.md",
    ];
    let combined = technical_docs
        .iter()
        .map(|path| std::fs::read_to_string(path).expect("technical doc is readable"))
        .collect::<Vec<_>>()
        .join("\n");

    for stale in [
        "xmind_cli_core",
        "validate_after",
        "inspect.rs",
        "read.rs",
        "mutate.rs",
        "topic-metadata.xmind",
        "pulldown-cmark",
        "miette",
        "cargo audit",
        "cargo doc --workspace --no-deps",
    ] {
        assert!(
            !combined.contains(stale),
            "technical docs still mention stale implementation detail `{stale}`"
        );
    }

    let crate_layout = std::fs::read_to_string("docs/technical/crate-layout.md")
        .expect("crate layout is readable");
    for module in ["patch.rs", "set.rs", "set_image.rs", "tree_input.rs"] {
        assert!(
            crate_layout.contains(module),
            "crate layout should document implemented app module `{module}`"
        );
    }
    assert!(
        combined.contains("./scripts/quality-gate.sh"),
        "technical docs should point to the implemented local quality gate"
    );
}

#[test]
fn xmind_fixture_manifest_covers_committed_workbooks_and_governance() {
    let manifest_path = "tests/fixtures/xmind/manifest.md";
    let manifest = std::fs::read_to_string(manifest_path).expect("fixture manifest is readable");

    for required in [
        "## Source Manifest",
        "## Governance Rules",
        "Fixture path",
        "Source",
        "Creation method",
        "Covered behavior",
        "PR gate",
        "Full matrix",
        "Mutation-safe copy strategy",
        "Privacy/license notes",
        "Regeneration status",
        "real-xmind-app",
        "synthetic",
        "each fixture under 1 MB",
        "total E2E fixture set under 10 MB",
    ] {
        assert!(
            manifest.contains(required),
            "{manifest_path} should document `{required}`"
        );
    }

    let mut fixture_paths = Vec::new();
    collect_xmind_fixtures(
        std::path::Path::new("tests/fixtures/xmind"),
        &mut fixture_paths,
    );
    for path in fixture_paths {
        let path = path.to_string_lossy();
        assert!(
            manifest.contains(path.as_ref()),
            "{manifest_path} should inventory committed fixture `{path}`"
        );
    }

    let malformed_row = manifest
        .lines()
        .find(|line| line.contains("tests/fixtures/xmind/malformed.xmind"))
        .expect("malformed fixture is inventoried");
    assert!(
        malformed_row.contains("synthetic"),
        "malformed fixture should be labeled synthetic so it is not mistaken for a representative user file"
    );

    let e2e_plan =
        std::fs::read_to_string("docs/technical/e2e-test-plan.md").expect("E2E plan is readable");
    assert!(
        e2e_plan.contains(manifest_path),
        "E2E plan should link to the fixture manifest"
    );

    let fixture_readme = std::fs::read_to_string("tests/fixtures/xmind/README.md")
        .expect("fixture README is readable");
    assert!(
        fixture_readme.contains(manifest_path),
        "fixture README should link to the fixture manifest"
    );
}

#[test]
fn fixture_manifest_records_real_app_fixture_and_followups() {
    let manifest_path = "tests/fixtures/xmind/manifest.md";
    let manifest = std::fs::read_to_string(manifest_path).expect("fixture manifest is readable");

    for required in [
        "## Issue #11 Real XMind App Fixture Evidence and Follow-ups",
        "tests/fixtures/xmind/real-app/real-app-fixture.xmind",
        "real-xmind-app",
        "Real App Fixture",
        "/Applications/Xmind.app",
        "CFBundleShortVersionString",
        "26.02.04171",
        "net.xmind.vana.app",
        "sdef: couldn't get sdef",
        "Computer Use",
        "xmind inspect",
        "xmind tree",
        "xmind validate",
    ] {
        assert!(
            manifest.contains(required),
            "{manifest_path} should preserve real-app handoff evidence `{required}`"
        );
    }
}

fn collect_xmind_fixtures(dir: &std::path::Path, output: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("fixture directory is readable") {
        let path = entry.expect("fixture entry is readable").path();
        if path.is_dir() {
            collect_xmind_fixtures(&path, output);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("xmind") {
            output.push(path);
        }
    }
}

#[test]
fn installation_docs_cover_supported_install_paths() {
    let install_doc =
        std::fs::read_to_string("docs/installation.md").expect("installation doc is readable");

    for expected in [
        "cargo install --locked --git https://github.com/ivan-94/xmind-cli",
        "cargo install --path .",
        "cargo build --workspace --release",
        "target/release/xmind",
        "bash scripts/install.sh --dry-run --version v0.1.0",
        "SHA256SUMS",
        "xmind completion bash",
        "xmind --version",
    ] {
        assert!(
            install_doc.contains(expected),
            "installation doc should mention `{expected}`"
        );
    }
}

#[test]
fn changelog_tracks_current_release_hardening_work() {
    let changelog = std::fs::read_to_string("CHANGELOG.md").expect("changelog is readable");

    for expected in [
        "## Unreleased",
        "CI",
        "shell completion",
        "installation",
        "release build",
    ] {
        assert!(
            changelog.contains(expected),
            "changelog should mention `{expected}`"
        );
    }
}

#[test]
fn root_readme_publishes_repository_baseline_without_overclaiming_release_channels() {
    let readme = std::fs::read_to_string("README.md").expect("root README is readable");

    for expected in [
        "# xmind-cli",
        "unofficial",
        "AI",
        "[中文](README.zh-CN.md)",
        "0.1.0",
        "publish = false",
        "Unreleased",
        "docs/reference/cli-overview.md",
        "docs/technical/release-policy.md",
        "CHANGELOG.md",
        "docs/installation.md",
        "cargo install --locked --git https://github.com/ivan-94/xmind-cli",
        "xmind completion <shell>",
        "xmind tree tests/fixtures/xmind/minimal.xmind --depth 2 --json",
        "MIT",
    ] {
        assert!(
            readme.contains(expected),
            "root README should include `{expected}`"
        );
    }

    let homebrew_overclaim = "brew install ivan-94/tap/xmind-cli";
    assert!(
        !readme.contains(homebrew_overclaim),
        "root README should not claim unreleased install channel `{homebrew_overclaim}` is available"
    );

    assert!(
        readme.contains("xmind add-tree /tmp/roadmap.xmind \\\n  --parent \"path:/Q2\" \\\n  --input docs/examples/simple-tree.yaml \\\n  --apply \\\n  --backup \\\n  --json"),
        "root README should present implemented `add-tree --apply --backup` safe edit workflow"
    );
    assert!(
        !readme.contains("`add-tree --apply` is planned as part of PRD #1 issue #18"),
        "root README should not describe implemented `add-tree --apply` as a planned follow-up"
    );
}

#[test]
fn release_policy_documents_versioning_changelog_notes_and_checksums() {
    let release_policy = std::fs::read_to_string("docs/technical/release-policy.md")
        .expect("release policy is readable");
    let installation_doc =
        std::fs::read_to_string("docs/installation.md").expect("installation doc is readable");
    let docs_readme = std::fs::read_to_string("docs/README.md").expect("docs readme is readable");

    for expected in [
        "v0.1.0",
        "Cargo.toml",
        "CHANGELOG.md",
        "GitHub Release",
        "SHA256SUMS",
        "shasum -a 256 -c SHA256SUMS",
        "crates.io",
    ] {
        assert!(
            release_policy.contains(expected),
            "release policy should mention `{expected}`"
        );
    }

    for expected in ["SHA256SUMS", "shasum -a 256 -c SHA256SUMS"] {
        assert!(
            installation_doc.contains(expected),
            "installation doc should mention checksum verification with `{expected}`"
        );
    }

    assert!(
        docs_readme.contains("technical/release-policy.md"),
        "docs README should link to the release policy"
    );
}

#[test]
fn cli_overview_keeps_diff_scope_aligned_with_current_command_contract() {
    let overview =
        std::fs::read_to_string("docs/reference/cli-overview.md").expect("overview is readable");

    assert!(
        overview.contains("current single-workbook diff surface"),
        "CLI overview should describe the implemented diff surface"
    );
    for stale in ["compare workbooks", "preview operation diffs"] {
        assert!(
            !overview.contains(stale),
            "CLI overview should not overclaim unsupported diff behavior `{stale}`"
        );
    }
}
