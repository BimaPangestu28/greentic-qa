# QA-PR-06 — component-qa skeleton (WASM + WIT) + basic operations

> **Goal:** component-qa skeleton (WASM + WIT) + basic operations

## Summary

Introduce `component-qa` as a WASM component (wasm32-wasip2), with WIT interface and a minimal runtime.

This PR wires `component-qa` to `qa-spec` and exposes "describe + schema + examples + validate" operations.

## Deliverables

- `crates/component-qa/wit/qa.wit` (or `wit/world.wit`)
- `crates/component-qa/src/lib.rs` implementing exported functions
- component build target config (`crate-type = ["cdylib"]` as needed for component toolchain)
- minimal host-agnostic logic; no rendering yet

## WIT design (minimum)

Interface `qa`:
- `get_form_spec(form_id: string) -> string`  (returns JSON)
- `get_answer_schema(form_id: string, ctx_json: string) -> string`
- `get_example_answers(form_id: string, ctx_json: string) -> string`
- `validate_answers(form_id: string, ctx_json: string, answers_json: string) -> string` (returns ValidationResult JSON)

Notes:
- Use JSON string payloads for MVP to reduce WIT churn
- Later PRs can introduce typed records

## FormSpec loading strategy (MVP)

Support at least one:
- spec is passed via config (`config.form_spec_json`)
- OR spec loaded from embedded asset path (pack mounting) — if supported in your environment

Document the chosen approach.

## Tests

- unit tests in Rust for core functions (non-wasm)
- optionally: wasm smoke test harness can come later (PR-15)

## Acceptance criteria

- component builds for wasm32-wasip2
- basic API returns expected JSON outputs for a fixture spec

