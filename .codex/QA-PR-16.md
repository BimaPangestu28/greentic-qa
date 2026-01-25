# QA-PR-16 — Nightly E2E tests: build pack + simulate wizard run

> **Goal:** Nightly E2E tests: build pack + simulate wizard run

## Summary

Nightly E2E tests for:
- building `qa-wizard-pack`
- simulating wizard execution headlessly
- validating that emitted files are correct and stable

## Deliverables

- `crates/qa-e2e/` (optional) or tests under `qa-cli/tests/e2e/`
- `nightly-e2e.yml` updated to:
  - build pack
  - run simulation with deterministic answers
  - compare generated outputs against snapshots
  - validate Adaptive Card JSON contains version 1.3 and supported elements only

## Headless simulation approach

- Invoke `component-qa` render/submit loop programmatically:
  - start with empty AnswerSet
  - render next step (card or json)
  - submit predetermined answers
  - repeat until complete
- Capture `qa.wizard.generated` event payload
- Materialize into temp dir and diff against snapshot fixtures

## Acceptance criteria

- Nightly job is stable and deterministic
- Failures produce actionable diffs (changed schema, changed card JSON, etc.)

