# QA-PR-02 — CI foundation: local_check + GitHub workflows (fast + parallel)

> **Goal:** CI foundation: local_check + GitHub workflows (fast + parallel)

## Summary

Add efficient GitHub workflows:
- `ci.yml` for PRs and pushes (fmt/clippy/test in parallel)
- `publish.yml` for publishing + binstall verification + smoke tests (in parallel where possible)
- `nightly-e2e.yml` scheduled for end-to-end tests

This PR sets up the pipeline structure. Some steps may be temporarily “no-op” until later PRs add real artifacts; but the workflow should be correct and ready.

## Deliverables

### New files
- `.github/workflows/ci.yml`
- `.github/workflows/publish.yml`
- `.github/workflows/nightly-e2e.yml`
- `ci/scripts/` helpers (if needed):
  - `ci/scripts/install_rust.sh` (optional, prefer actions-rs/toolchain)
  - `ci/scripts/smoke.sh` (placeholder now, real later)
  - `ci/scripts/binstall_verify.sh` (placeholder now, real later)

## Workflow design

### `ci.yml`
Triggers:
- `pull_request`
- `push` to `master`

Jobs (parallel):
1) `fmt` — `cargo fmt --check`
2) `clippy` — `cargo clippy --workspace --all-targets -- -D warnings`
3) `test` — `cargo test --workspace`

Each job:
- checks out repo
- installs toolchain (cache)
- uses cargo cache action

### `publish.yml`
Trigger:
- `push` to `master` **only** (and/or tags if you prefer)

Gating:
- `needs: [fmt, clippy, test]` by reusing `ci.yml` outputs (or duplicate steps)

Jobs (parallel after gating):
1) `publish_crates` — publish crates to crates.io using `CARGO_REGISTRY_TOKEN`
2) `binstall_matrix` — verify installation on:
   - ubuntu-latest
   - windows-latest
   - macos-15 (intel)
   - macos-15 (arm)  *(use appropriate runner labels supported by GitHub)*
3) `smoke` — installs binaries, runs `qa-cli --help` and a minimal `qa-cli new ...` once implemented

Note: until later PRs implement `qa-cli`, smoke may only build and run `--help`.

### `nightly-e2e.yml`
Trigger:
- schedule (e.g. daily 02:00 UTC)
- `workflow_dispatch`

Steps:
- build workspace
- run E2E suite (to be filled in PR-16)
- for now: run a placeholder `cargo test -p qa-cli` or `-p qa-spec` to validate plumbing

## Acceptance criteria

- Workflows are syntactically valid.
- CI runs fmt/clippy/test in parallel.
- Publish workflow is present and gated on CI success.
- Nightly workflow exists and runs successfully (even if minimal).

