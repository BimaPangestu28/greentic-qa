# QA-PR-07 — Progress engine + storage mapping (state/config/payload_out)

> **Goal:** Progress engine + storage mapping (state/config/payload_out)

## Summary

Implement progress engine and declarative storage mapping.

- `next()` selects next visible unanswered question (FormSpec)
- skip/default policies implemented
- `store[]` applies answer values into answers/state/config/payload_out (as patches)

## Deliverables

### In qa-spec
- `progress.rs`
  - `next_question(form, ctx, answers) -> Option<QuestionId>`
  - policy evaluation

- `store.rs`
  - apply a list of `StoreOp`:
    - `{ target: answers|state|config|payload_out, path: json_pointer, value: template_or_literal }`

### In component-qa
- expose:
  - `next(form_id, ctx_json, answers_json) -> string`
  - `apply_store(form_id, ctx_json, answers_json) -> string` (returns patches or updated ctx)

## Policies (MVP)

- `skip_answered: bool`
- `autofill_defaults: bool`
- `treat_default_as_answered: bool`

Per-question override:
- `skip_if_present_in: ["answers","config","state"]`
- `editable_if_from_default: bool`

## Tests

- skip answered works
- default autfill works
- visibility impacts next selection
- store mapping writes correct JSON paths (create objects)

## Acceptance criteria

- deterministic `next` behavior across fixtures
- store mapping produces correct patches

