# QA-PR-05 — Answer schema + example generation + validation engine

> **Goal:** Answer schema + example generation + validation engine

## Summary

Implement:
- answer JSON schema generation from `FormSpec`
- example answers generation
- validation engine with structured errors

## Key functions

- `answers_schema::generate(&FormSpec, resolved_visibility: ...) -> serde_json::Value`
- `examples::generate(&FormSpec) -> serde_json::Value`
- `validate::validate(&FormSpec, answers: &serde_json::Value) -> ValidationResult`

`ValidationResult`:
- `valid: bool`
- `errors: Vec<ValidationError>`
- `missing_required: Vec<String>`
- `unknown_fields: Vec<String>`

## Visibility and "required"
MVP rule:
- if a question is not visible under current answers/context, it is not required.
- visibility is computed by evaluating `visible_if` expressions using a lightweight evaluator.
- if visibility cannot be evaluated (missing dependency), treat as visible (conservative) OR treat as hidden (strict). Pick one, document.

## Constraints supported (MVP)
- types: string, boolean, integer, number, enum
- regex pattern for strings
- min/max for numbers
- minLen/maxLen for strings
- required

## Tests
- fixtures:
  - valid minimal answers
  - missing required
  - invalid pattern
  - enum mismatch
  - unknown field strict mode
  - conditional visibility required

Golden snapshots:
- schema JSON
- example answers JSON

## Acceptance criteria
- schema generation stable and correct for fixtures
- validation returns actionable error paths and question ids
- tests pass

