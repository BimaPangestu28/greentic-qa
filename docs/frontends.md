# Frontends

This repository supports three frontend render targets over the same render payload:

1. Text UI
- Human-readable console output.
- Used by CLI wizard presenter.

2. JSON UI
- Structured JSON payload for machine-driven or external orchestration flows.

3. Adaptive Card UI
- Adaptive Card v1.3 JSON transport suitable for card-capable hosts.

## Abstraction
- `qa-spec` exposes `QaFrontend` and `DefaultQaFrontend` wrappers.
- Existing renderer behavior is reused; output compatibility is preserved.

## Determinism
- Rendering reads `RenderPayload` only.
- State mutation and store side effects are outside frontend rendering responsibilities.

## Component payload compatibility
- Single endpoint version is used (no parallel `next2`-style API).
- `component-qa` accepts both:
  - legacy direct payloads
  - additive envelopes (`form_spec_json`, `include_registry`, optional `ctx`)
- Unknown fields are ignored for forward compatibility.

## Adaptive card debug metadata
- Default card output stays user-facing and clean.
- When runtime context sets `i18n_debug: true` (or `debug_i18n: true`), card output adds:
  - `metadata.qa.i18n_debug`
  - `metadata.qa.questions[*].title_key` / `description_key`
