# QA-PR-13 — qa-wizard-pack: deployable gtpack driven by component-qa cards

> **Goal:** qa-wizard-pack: deployable gtpack driven by component-qa cards

## Summary

Add deployable gtpack `qa-wizard-pack` that provides an Adaptive Card–driven wizard (card transport) to generate QA specs.

This pack is intended to be deployed in environments where authoring should happen via card UI.

## Pack contents

- `packs/qa-wizard-pack/pack.yaml`
- `packs/qa-wizard-pack/pack.lock` (or json) as required by greentic tooling
- `packs/qa-wizard-pack/flows/qa_wizard.flow.yaml` (or your flow format)
- `packs/qa-wizard-pack/assets/` (optional)
- `packs/qa-wizard-pack/README.md`

## Wizard behavior (MVP)

- Wizard implemented as QAFlowSpec executed by `component-qa` (card mode).
- Steps:
  1) basics: id/title/version
  2) intro text
  3) questions: add one question; loop until done
  4) policies
  5) generate outputs (in-memory)
  6) emit an event `qa.wizard.generated` with the generated files

## Output payload

Event data includes:
- `dir_name: "my-provider-setup"`
- `files: [{ path, contents_base64, content_type }]`
- `summary_md`

## Tests

- pack builds (doctor/build as applicable)
- flow loads and produces deterministic event payload in headless simulation (to be expanded in PR-16)

## Acceptance criteria

- pack directory exists and validates
- wizard can run through a minimal path and emit generation event

