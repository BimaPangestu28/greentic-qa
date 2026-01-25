# QA-PR-14 — Wizard pack persistence options (events default + dev writer)

> **Goal:** Wizard pack persistence options (events default + dev writer)

## Summary

Add persistence options for wizard outputs.

Default recommended: **emit events only**.
Add optional dev-mode file writer for local testing.

## Options

### A) Event-only (default)
- pack emits `qa.wizard.generated`
- downstream tool/component decides where to store

### B) Workspace writer (dev)
- if `QA_WIZARD_OUTPUT_DIR` is configured (or config key), write files to that directory
- include safety checks:
  - must be under an allowed root
  - must not overwrite existing files unless `--force` set

### C) Git writer (deferred)
- explicitly NOT implemented in this PR, but define interface event types

## Deliverables

- `qa-wizard-pack` config schema describing persistence mode
- `component-qa` or small helper crate `qa-persist` (optional) to implement dev writer
- document usage

## Tests

- unit test for path safety and directory creation
- integration test in `qa-cli` to consume emitted event and write to disk (optional)

## Acceptance criteria

- default behavior unchanged (events only)
- dev-mode writer works and is safe by default

