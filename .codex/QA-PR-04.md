# QA-PR-04 — qa-spec crate: handlebars templating context + helpers

> **Goal:** qa-spec crate: handlebars templating context + helpers

## Summary

Add Handlebars templating helpers and define a canonical template context:
`{ payload, state, config, answers, secrets? }`

This PR provides:
- helper registration
- strict vs relaxed modes
- function to "resolve" templated fields in a spec (without mutating originals unless asked)

## Deliverables

### New module
- `crates/qa-spec/src/template/mod.rs`
  - `TemplateEngine`
  - `TemplateContext`
  - `register_default_helpers()`

### Default helpers
- `get path default?`
- `default a b`
- `eq a b`
- `and a b`
- `or a b`
- `not a`
- `len x`
- `json x` (stringify JSON)

## Resolution behavior

- Only string fields are templated.
- Provide `resolve_form_spec(&FormSpec, &TemplateContext) -> ResolvedFormSpec` (or same type)
- Provide `resolve_string(&str, &TemplateContext) -> Result<String>`

Strict mode:
- missing keys -> error

Relaxed mode:
- missing keys -> empty string or keep handlebars token (pick one and document)

## Tests

- helper behavior tests
- templating in:
  - question titles/descriptions
  - defaults
  - intro text

## Acceptance criteria

- `qa-spec` can template strings deterministically.
- No secrets are supported yet beyond placeholder; secrets gating comes later.

