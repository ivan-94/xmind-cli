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
