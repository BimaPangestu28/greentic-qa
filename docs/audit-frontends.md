# Frontend Audit (PR-QA-01)

## Repository map
- `crates/qa-spec`: core spec, validation, rendering, progress.
- `crates/component-qa`: component transport endpoints and compatibility APIs.
- `crates/qa-cli`: CLI commands (`wizard`, `generate`, `validate`) and text presenter.
- `packs/qa-wizard-pack`: flow packaging artifacts.

## Current extension points
- Rendering payload construction:
  - `crates/qa-spec/src/render.rs` (`build_render_payload`)
- Existing frontend renderers:
  - text/json/adaptive card in `crates/qa-spec/src/render.rs`
- CLI text presentation:
  - `crates/qa-cli/src/wizard.rs`
- Component submit/apply boundary:
  - `crates/component-qa/src/lib.rs` (`submit_patch`, `submit_all`, `apply_store`)

## Safety model notes
- Bundle generation writes are env-gated in CLI (`QA_WIZARD_OUTPUT_DIR`, `QA_WIZARD_ALLOWED_ROOTS`).
- Wizard/renderer flows are response-oriented and do not write files directly.
- Store operations in component are executed via `StoreContext::apply_ops`.

## PR-QA-01 chosen path
- Keep current render outputs and wrap with frontend trait (`QaFrontend`).
- Enforce deterministic plan-first boundary:
  - planning in `qa-spec` runner (pure)
  - execution in component adapter (`component-qa`)
- Keep `submit_patch` as compatibility wrapper.
- Stage broader WIT signature changes to follow-up PR.
