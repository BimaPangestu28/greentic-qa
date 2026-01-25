# Repo Overview

## What this repo contains

- `crates/qa-spec`: the canonical FormSpec/QAFlowSpec/AnswerSet model bundle shared across components.
- `crates/component-qa`: a wasm32-wasip2 component (via `greentic-component`) that exposes spec/schema/examples/validation/render helpers.
- `crates/qa-cli`: text-based wizard for authoring specs, validating answers, and driving the wizard pack.
- `packs/qa-wizard-pack`: deployable gtpack that wires `component-qa` cards into an authoring flow; the pack is built with `greentic-pack`/`greentic-flow`.

## Running checks locally

1. `ci/local_check.sh` runs fmt, clippy, and all workspace tests. It mirrors the PR `ci.yml` jobs.
2. Each crate can also be built individually (`cargo test -p qa-spec`, `cargo test -p qa-cli`, etc.).
3. Additional validation scripts (publish/binstall/nightly-e2e) are described in `.codex/QA-PR-02.md` and downstream PRs.

## PR series organization

- QA-PR-01/02 bootstrap the repo and CI infrastructure.
- QA-PR-03 through QA-PR-11 build the spec models, templating, secrets, storage/policies, renderers, and submit handlers.
- QA-PR-12 through QA-PR-16 cover the qa-cli wizard, deployable pack, persistence options, publishing flows, and E2E automation.

Refer to the `.codex/QA-PR-*.md` files for detailed tasks per PR.
