# xmind-cli

[English](README.md)

`xmind-cli` 是一个非官方命令行工具，用于检查、查询、编辑、校验和导出
XMind 工作簿。仓库名是 `xmind-cli`，安装后的可执行命令是 `xmind`。

这个项目面向人类和 AI 协作场景：当思维导图需要像结构化数据一样被读取、
定位、预览修改和校验时，`xmind` 可以让 Agent 先检查工作簿，用稳定 selector
定位主题，通过 JSON diff 预览变更，再显式应用修改并校验结果。

本项目与 XMind 官方没有从属、背书或赞助关系。

## 当前状态

`xmind-cli` 仍处于早期发布阶段。当前包版本是 `0.1.0`，`Cargo.toml` 中
`publish = false`，`CHANGELOG.md` 已记录当前 `v0.1.0` release。当前 CLI 可以从源码
安装或构建，并且已为已实现的命令面提供集成测试。带 tag 的 release 会发布 GitHub
Release binaries、安装脚本和 Homebrew tap formula。

对真实工作簿使用写入命令前，请先运行 dry run，检查 JSON 或人类可读 diff，并
保留原始文件备份。

## 核心能力

- 检查工作簿结构、sheet、topic、元数据和校验状态。
- 通过标题、id 或 path 风格 selector 查找并读取 topic。
- 使用 `--dry-run` 和 `--json` 预览安全修改。
- 使用显式 `--apply` 应用已支持的 topic 和子树编辑。
- 创建备份，并从匹配的 `.xmind-backups` 条目恢复。
- 按命令参考中记录的能力导出工作簿内容。
- 使用 `xmind completion <shell>` 生成 shell 补全脚本。

## 安装或构建

从当前 GitHub 源码安装：

```bash
cargo install --locked --git https://github.com/ivan-94/xmind-cli
```

从本地 checkout 构建：

```bash
cargo build --workspace --release
target/release/xmind --version
```

开发时直接运行：

```bash
cargo run -- tree tests/fixtures/xmind/minimal.xmind --depth 2
```

带 tag 的 GitHub Release 归档、安装脚本和 Homebrew tap 都属于发布流程的一部分。
当前安装、release build、checksum 和 shell 补全说明见
[docs/installation.md](docs/installation.md)。

| 渠道 | 状态 | 说明 |
| --- | --- | --- |
| 从 checkout 进行 Cargo 源码安装 | 当前可用 | `cargo install --path .` |
| 从 GitHub 进行 Cargo 源码安装 | 当前可用 | `cargo install --locked --git https://github.com/ivan-94/xmind-cli` |
| 本地 release build | 当前可用 | `cargo build --workspace --release`，然后运行 `target/release/xmind` |
| GitHub Release binaries | 首个 tagged release 后可用 | 计划目标：macOS Apple Silicon、macOS Intel、Linux x86_64 GNU、Linux arm64 GNU、Windows x86_64 MSVC。 |
| Install script | 面向 tagged release artifacts | 先运行 `bash scripts/install.sh --dry-run --version v0.1.0`，确认后去掉 `--dry-run`。 |
| Homebrew tap | tagged release 后可用 | `brew install ivan-94/tap/xmind-cli` |

第一版二进制 release 矩阵不代表支持：Linux musl/static builds、macOS universal binaries、32-bit Windows、Windows GNU、container images 或 crates.io packages。

## 快速开始

检查已提交 fixture：

```bash
xmind inspect tests/fixtures/xmind/minimal.xmind --json
xmind sheets tests/fixtures/xmind/minimal.xmind --json
xmind tree tests/fixtures/xmind/minimal.xmind --depth 2 --json
```

查找并读取 topic：

```bash
xmind find tests/fixtures/xmind/minimal.xmind --title "Payment" --json
xmind get tests/fixtures/xmind/minimal.xmind --node "path:/Q2/Payment" --json
```

写入前先预览子树编辑：

```bash
cp tests/fixtures/xmind/minimal.xmind /tmp/roadmap.xmind
xmind add-tree /tmp/roadmap.xmind \
  --parent "path:/Q2" \
  --input docs/examples/simple-tree.yaml \
  --dry-run \
  --json
```

只有当 dry-run 输出符合预期时，再应用修改：

```bash
xmind add-tree /tmp/roadmap.xmind \
  --parent "path:/Q2" \
  --input docs/examples/simple-tree.yaml \
  --apply \
  --backup \
  --json
xmind validate /tmp/roadmap.xmind --json
```

## 安全模型

- 写入命令必须且只能选择 `--dry-run` 或 `--apply` 之一。
- Dry run 只计算计划变更，不写入文件系统。
- Apply 路径会在替换原文件前校验工作簿。
- `--backup` 会为原地修改创建带时间戳的备份。
- selector 有歧义时会失败，不会猜测目标。
- JSON 输出遵循已记录的成功和错误 envelope contract。

详细规则见 [mutation semantics](docs/reference/mutation-semantics.md)、
[output formats](docs/reference/output-formats.md) 和
[agent error contract](docs/reference/agent-error-contract.md)。

## 文档

- [文档地图](docs/README.md)
- [安装说明](docs/installation.md)
- [命令参考](docs/reference/cli-overview.md)
- [快速开始指南](docs/guides/quick-start.md)
- [Agent recipes](docs/guides/agent-recipes.md)
- [安全编辑流程](docs/guides/safe-editing-workflow.md)
- [Mutation semantics](docs/reference/mutation-semantics.md)
- [发布策略](docs/technical/release-policy.md)
- [更新日志](CHANGELOG.md)

## 质量检查

分享变更前运行本地质量门禁：

```bash
./scripts/quality-gate.sh
```

聚焦文档检查可以运行：

```bash
cargo test --test doc_examples_test
git diff --check
```

CI workflow 会覆盖 formatting、clippy、tests、docs build、release build smoke、
`cargo audit` 和 `cargo deny`。

[![CI](https://github.com/ivan-94/xmind-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/ivan-94/xmind-cli/actions/workflows/ci.yml)

## 贡献和支持

本仓库仍在快速演进中，小而聚焦的变更比大范围重写更容易审查。报告问题时，请提供
你运行的命令、是否使用 `--json`、退出码，以及可复现的最小工作簿或 fixture 路径。

修改命令行为时，请同步更新相关文档和测试，确保 README 示例、命令参考和 CLI 行为
保持一致。

## 许可证

`Cargo.toml` 声明本项目使用 MIT license。
