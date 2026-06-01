# CLAUDE.md

Guidance for AI assistants (Claude Code and others) working in this repository.

> **Authoritative conventions live in [`AGENTS.md`](./AGENTS.md).** This file
> summarizes the repo and points to the right places; when editing files in a
> subtree, always read the nearest `AGENTS.md` (there are additional ones in
> [`codex-rs/tui/src/bottom_pane/`](./codex-rs/tui/src/bottom_pane/AGENTS.md)
> and [`codex-rs/thread-store/src/remote/`](./codex-rs/thread-store/src/remote/AGENTS.md)).

## What this repo is

This is the [OpenAI Codex CLI](https://github.com/openai/codex) — a coding agent
that runs locally on the user's machine. It is distributed as `@openai/codex`
(npm) and via Homebrew. **All current CLI development happens in Rust** under
[`codex-rs/`](./codex-rs); the npm/Homebrew packages ship the compiled Rust
binaries. There are also language SDKs under [`sdk/`](./sdk).

## Repository layout

| Path | Purpose |
| --- | --- |
| [`codex-rs/`](./codex-rs) | The Rust Cargo workspace — the heart of the project (~80 crates). All agent logic, the CLIs, sandboxing, and protocols live here. |
| [`codex-cli/`](./codex-cli) | npm package `@openai/codex`: a thin Node wrapper (`bin/`) that ships the compiled Rust binaries, plus packaging `scripts/`. |
| [`sdk/`](./sdk) | Language SDKs that drive Codex programmatically: [`typescript/`](./sdk/typescript), [`python/`](./sdk/python), and `python-runtime/`. |
| [`docs/`](./docs) | User- and contributor-facing documentation (see below). |
| [`scripts/`](./scripts) | Repo-level helper scripts (npm staging, README TOC, blob-size checks, etc.). |
| [`.devcontainer/`](./.devcontainer) | Dev container definition. |
| [`.github/workflows/`](./.github/workflows) | CI pipelines. |
| `third_party/`, `tools/`, `patches/`, `vendor/` | Vendored deps and build tooling. |

Top-level build/config files worth knowing: `justfile` (the `just` command
runner — see below), `package.json` + `pnpm-workspace.yaml` + `pnpm-lock.yaml`
(JS workspace), Bazel (`MODULE.bazel`, `BUILD.bazel`, `.bazelrc`, `defs.bzl`),
Nix (`flake.nix`), and `cliff.toml` (git-cliff changelog generation).

### Key Rust crates (under `codex-rs/`)

Crate names are prefixed with `codex-`; e.g. the `core/` folder builds the
`codex-core` crate. The workspace is large; the most important crates:

- [`core/`](./codex-rs/core) — `codex-core`, the UI-agnostic business logic
  (conversation/turn management, model clients, tool execution, config,
  sandboxing glue). The largest crate — **resist adding new code here**
  (see "Conventions").
- [`cli/`](./codex-rs/cli) — the `codex` multitool binary. Running `codex` with
  no subcommand launches the interactive TUI; subcommands cover headless
  execution and integrations (e.g. `exec`, MCP, the app server, login).
- [`tui/`](./codex-rs/tui) — the fullscreen terminal UI built with
  [Ratatui](https://ratatui.rs/). Has its own style and code conventions
  (see `codex-rs/tui/styles.md` and `AGENTS.md`).
- [`exec/`](./codex-rs/exec), [`exec-server/`](./codex-rs/exec-server) —
  headless/automation execution.
- [`mcp-server/`](./codex-rs/mcp-server), [`codex-mcp/`](./codex-rs/codex-mcp),
  [`rmcp-client/`](./codex-rs/rmcp-client) — Model Context Protocol support.
- [`app-server/`](./codex-rs/app-server),
  [`app-server-protocol/`](./codex-rs/app-server-protocol),
  [`app-server-client/`](./codex-rs/app-server-client) — the app server used by
  editor/IDE integrations and the SDKs. **All new API development happens in
  app-server v2; do not extend v1.**
- [`protocol/`](./codex-rs/protocol) — the wire protocol types.
- Sandboxing: [`linux-sandbox/`](./codex-rs/linux-sandbox),
  [`windows-sandbox-rs/`](./codex-rs/windows-sandbox-rs),
  [`sandboxing/`](./codex-rs/sandboxing),
  [`execpolicy/`](./codex-rs/execpolicy).
- [`apply-patch/`](./codex-rs/apply-patch) — the patch format Codex emits/applies.
- [`login/`](./codex-rs/login), [`chatgpt/`](./codex-rs/chatgpt) — auth flows.

### Documentation

Useful docs in [`docs/`](./docs): [`getting-started.md`](./docs/getting-started.md),
[`install.md`](./docs/install.md), [`config.md`](./docs/config.md),
[`authentication.md`](./docs/authentication.md), [`sandbox.md`](./docs/sandbox.md),
[`exec.md`](./docs/exec.md), [`execpolicy.md`](./docs/execpolicy.md),
[`skills.md`](./docs/skills.md), [`slash_commands.md`](./docs/slash_commands.md),
[`agents_md.md`](./docs/agents_md.md), and [`contributing.md`](./docs/contributing.md).

## Development workflow (Rust / `codex-rs`)

The workspace uses **Rust 1.93.0** (pinned in
[`codex-rs/rust-toolchain.toml`](./codex-rs/rust-toolchain.toml)) and **edition
2024** (`resolver = "2"`). It builds with both **Cargo** and **Bazel**
(`just bazel-*` recipes / `bazel build //codex-rs/...`); Cargo is the day-to-day
path.

Use the [`just`](https://github.com/casey/just) command runner — the
[`justfile`](./justfile) lives at the repo root but runs recipes from
`codex-rs/`, so invoke `just <recipe>` from the repo root (or `codex-rs/`).
Install any tools the repo relies on (e.g. `just`, `rg`, `cargo-insta`,
`cargo-nextest`) if they aren't already available. Key recipes:

| Command | What it does |
| --- | --- |
| `just fmt` | Format Rust code. **Run automatically after finishing Rust changes — do not ask for approval.** |
| `just fix -p <crate>` | `cargo clippy --fix` scoped to one crate. Run before finalizing a large change. Prefer `-p` to avoid slow workspace-wide Clippy. |
| `just test` | Run the test suite (uses `cargo-nextest` when available). |
| `just write-config-schema` | Regenerate `codex-rs/core/config.schema.json` after changing `ConfigToml` or nested config types. |
| `just write-app-server-schema` | Regenerate app-server schema fixtures after changing API shapes (`--experimental` for experimental fixtures). |
| `just bazel-lock-update` / `just bazel-lock-check` | Refresh / verify `MODULE.bazel.lock` after changing Cargo deps. |
| `just argument-comment-lint` | Run the positional-argument-comment lint (Bazel-backed). |

Typical loop after editing Rust:

```bash
# 1. Run the test(s) for the crate you changed, e.g.:
cargo test -p codex-tui
# 2. If you touched common/core/protocol, run the full suite:
cargo test            # or: just test
# 3. Format and lint:
just fmt
just fix -p <crate-you-changed>
```

Notes:
- Be patient with Rust commands (`just fix`, `cargo test`) — lock contention can
  make them slow. **Do not kill them by PID.**
- Don't re-run tests after running `fix`/`fmt`.
- Ask before running the *full* `cargo test` suite; crate-scoped tests are fine
  to run without asking. Avoid `--all-features` for routine runs (it bloats
  `target/`).

### CI expectations

CI lives in [`.github/workflows/`](./.github/workflows) (`rust-ci.yml`,
`rust-ci-full.yml`, `bazel.yml`, `cargo-deny.yml`, `codespell.yml`, `sdk.yml`,
`blob-size-policy.yml`, …). Match these locally before pushing:

- **Format:** `cargo fmt -- --config imports_granularity=Item --check`
  (this is what `just fmt` writes).
- **Lint:** `cargo clippy --tests` with warnings denied (`just clippy`).
- **Unused deps:** `cargo shear` (run `cargo shear --fix` to clean up).
- **License/advisories:** `cargo deny` (config in `codex-rs/deny.toml`).
- **Spelling:** `codespell` (config in `.codespellrc`).
- **Argument-comment lint:** Bazel-based; run `just argument-comment-lint`
  (usually easier to let CI check it across all three platforms).

### TUI snapshot tests

The TUI uses [`insta`](https://insta.rs/) snapshot tests. **Any change that
affects user-visible UI must include snapshot coverage.** Workflow:

```bash
cargo test -p codex-tui                       # regenerate snapshots
cargo insta pending-snapshots -p codex-tui    # see what's pending
cargo insta accept -p codex-tui               # accept (only when intended)
```

Install with `cargo install cargo-insta` if missing.

## Development workflow (JS/TS SDK & npm package)

The JS workspace is managed with `pnpm` (`pnpm-workspace.yaml`). The TypeScript
SDK lives in [`sdk/typescript/`](./sdk/typescript); the distributable npm
package is built from [`codex-cli/`](./codex-cli). Formatting is Prettier
(`.prettierrc.toml`). Check each package's `package.json` for its `build`/`test`
scripts.

## Conventions (from `AGENTS.md` — read it in full)

These apply across the Rust codebase:

- **Crate names are prefixed with `codex-`.**
- **The repo is fully open source** — assume all comments and docs are public;
  write professional, concise documentation.
- **Inline variables into `format!`** — write `format!("{value}")`, not
  `format!("{}", value)`. Never write a bare URL in Markdown; always use a link.
- **Never add or modify code related to `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR`
  or `CODEX_SANDBOX_ENV_VAR`.** Tests use these to early-exit under the sandbox.
- Collapse nested `if`s; prefer method references over closures; make `match`
  statements exhaustive and avoid wildcard arms.
- Avoid `bool`/ambiguous `Option` parameters that produce opaque callsites like
  `foo(false)`; prefer enums/newtypes/named methods. When a positional literal
  is unavoidable, use the `/*param_name*/` comment convention.
- New traits get doc comments explaining their role and expected use.
- Prefer private modules with an explicit public crate API. Don't create small
  helper methods used only once.
- **Keep modules small:** target < 500 LoC (excluding tests); if a file exceeds
  ~800 LoC, add new functionality in a new module. This is especially important
  for high-touch files like `tui/src/app.rs`, `tui/src/chatwidget.rs`, and the
  `tui/src/bottom_pane/` modules.
- **Resist adding to `codex-core`** — prefer an existing more-specific crate, or
  introduce a new workspace crate, rather than growing `codex-core`.
- If you add `include_str!`/`include_bytes!`/`sqlx::migrate!` or similar
  build-time file reads, update the crate's `BUILD.bazel` so Bazel builds too.
- When changing an API, update the relevant docs in `docs/`.

### Testing conventions

- Use `pretty_assertions::assert_eq` and compare **whole objects**, not field by
  field.
- Don't mutate process env in tests; pass dependencies in from above.
- Prefer `codex_utils_cargo_bin::cargo_bin(...)` (and `find_resource!`) over
  `assert_cmd`/`escargot`/`CARGO_MANIFEST_DIR` so tests resolve correctly under
  both Cargo and Bazel runfiles.
- For end-to-end core tests, use the helpers in `core_test_support::responses`
  (e.g. `mount_sse_once`, the `ev_*` SSE constructors, `ResponseMock`).

### TUI styling

Prefer Ratatui `Stylize` helpers (`"text".dim()`, `.bold()`, `.cyan()`) over
hand-built `Span`/`Style`. Use `textwrap::wrap` for plain strings and the
helpers in `tui/src/wrapping.rs` / `tui/src/render/line_utils.rs` for `Line`s. See
`codex-rs/tui/styles.md`.

## Git / contribution

- Develop on a feature branch; open pull requests against `main`.
- Commit with clear, descriptive messages; lint and test before committing.
- See [`docs/contributing.md`](./docs/contributing.md) for the contribution
  process and CLA.
