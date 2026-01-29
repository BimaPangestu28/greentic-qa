# QA-PR-01 — Repo bootstrap + governance + Codex scaffolding

> **Goal:** Repo bootstrap + governance + Codex scaffolding

## Summary

Bootstrap a **new, production-ready GitHub repo** with:
- Codex scaffolding: `.codex/repo_overview.md`, `global_rules.md`
- Governance: `SECURITY.md`, `LICENSE.md` (MIT)
- Workspace skeleton with placeholder crates and pack dir
- Top-level `README.md` describing what this repo does

This PR should be mergeable with no functional QA logic yet, but everything should build and `ci/local_check.sh` should pass (even if it’s minimal at this stage).

## Deliverables

### New files
- `.codex/repo_overview.md`
- `global_rules.md`
- `SECURITY.md`
- `LICENSE.md` (MIT)
- `README.md`
- `.editorconfig`
- `.gitignore`
- `Cargo.toml` (workspace)
- `crates/qa-spec/Cargo.toml` + `src/lib.rs` (placeholder)
- `crates/component-qa/Cargo.toml` + `src/lib.rs` (placeholder)
- `crates/qa-cli/Cargo.toml` + `src/main.rs` (placeholder)
- `packs/qa-wizard-pack/README.md` (placeholder pack description)
- `ci/local_check.sh` (basic fmt/clippy/test)

### Crate metadata requirements (all crates)
Each `Cargo.toml` must include at least:
- `description`
- `license = "MIT"`
- `repository`
- `homepage` (or omit if not used, but be consistent)
- `edition = "2021"`
- `rust-version` (align with Greentic standard)

## Implementation notes

### Workspace `Cargo.toml`
- Use `resolver = "2"`
- Use `[workspace.package]` defaults (license, repository, edition) and inherit in crates via `workspace = true` where appropriate.

### `ci/local_check.sh`
- `set -euo pipefail`
- Run:
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`

### `.codex/repo_overview.md`
Include:
- What is `component-qa`, `qa-spec`, `qa-cli`, `qa-wizard-pack`
- How to run checks locally
- How the PR series is organized

### `global_rules.md`
Include:
- Formatting and lint rules
- Testing expectations
- “No interactive confirmations”: implement as much as possible; only ask for destructive operations.

## Tests

- `ci/local_check.sh` passes locally.
- `cargo test --workspace` runs (even if tests are minimal placeholders).

## Acceptance criteria

- Repo builds cleanly.
- All crates compile.
- `ci/local_check.sh` passes.
- Files listed above exist with sensible content and correct license headers where appropriate.

