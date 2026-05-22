use assert_cmd::Command;

struct DocExample {
    command: &'static str,
    args: &'static [&'static str],
}

const DOC_EXAMPLES: &[DocExample] = &[
    DocExample {
        command: "xmind inspect tests/fixtures/xmind/minimal.xmind --json",
        args: &["inspect", "tests/fixtures/xmind/minimal.xmind", "--json"],
    },
    DocExample {
        command: "xmind sheets tests/fixtures/xmind/minimal.xmind --json",
        args: &["sheets", "tests/fixtures/xmind/minimal.xmind", "--json"],
    },
    DocExample {
        command: "xmind tree tests/fixtures/xmind/minimal.xmind --depth 2 --json",
        args: &[
            "tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--depth",
            "2",
            "--json",
        ],
    },
    DocExample {
        command: "xmind get tests/fixtures/xmind/minimal.xmind --node path:/Q2/Payment --json",
        args: &[
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "path:/Q2/Payment",
            "--json",
        ],
    },
    DocExample {
        command: "xmind find tests/fixtures/xmind/minimal.xmind --title Payment --json",
        args: &[
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--title",
            "Payment",
            "--json",
        ],
    },
    DocExample {
        command: "xmind add-tree tests/fixtures/xmind/minimal.xmind --parent path:/Q2 --input docs/examples/simple-tree.yaml --dry-run --json",
        args: &[
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--dry-run",
            "--json",
        ],
    },
    DocExample {
        command: "xmind patch tests/fixtures/xmind/minimal.xmind --ops docs/examples/patch-add-tree.yaml --dry-run --json",
        args: &[
            "patch",
            "tests/fixtures/xmind/minimal.xmind",
            "--ops",
            "docs/examples/patch-add-tree.yaml",
            "--dry-run",
            "--json",
        ],
    },
];

#[test]
fn documented_fixture_examples_are_present_and_run() {
    let readme = std::fs::read_to_string("docs/examples/README.md")
        .expect("docs examples README is readable");

    for example in DOC_EXAMPLES {
        assert!(
            readme.contains(example.command),
            "docs/examples/README.md must include runnable example: {}",
            example.command
        );

        Command::cargo_bin("xmind")
            .expect("xmind binary is built for CLI tests")
            .args(example.args)
            .assert()
            .success();
    }
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
fn installation_docs_cover_supported_install_paths() {
    let install_doc =
        std::fs::read_to_string("docs/installation.md").expect("installation doc is readable");

    for expected in [
        "cargo install --path .",
        "cargo build --workspace --release",
        "target/release/xmind",
        "xmind completion bash",
        "xmind --version",
    ] {
        assert!(
            install_doc.contains(expected),
            "installation doc should mention `{expected}`"
        );
    }
}
