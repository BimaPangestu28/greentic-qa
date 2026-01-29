# QA-PR-11 — Submit handlers (patch + submit-all) + completion semantics

> **Goal:** Submit handlers (patch + submit-all) + completion semantics

## Summary

Implement submit handlers:
- patch: `submit_answer_patch(question_id, value)`
- all-at-once: `submit_answers_json(answers)`

Both must:
- validate
- commit into AnswerSet
- apply store mappings (state/config/payload_out and optionally secrets if allowed)
- return status + next question

## API design (component-qa)

- `submit_patch(form_id, ctx_json, answers_json, question_id, value_json) -> string`
- `submit_all(form_id, ctx_json, answers_json) -> string`

Return JSON:
- `status`
- `next_question_id?`
- `validation` details if error
- `patches` (state/config/payload_out updates) OR updated objects (choose one, recommend patches)

## Unknown fields behavior

- In patch mode: ignore unrelated existing answers (but validate only submitted field + overall consistency if needed)
- In all mode: if strict, unknown fields are errors; if permissive, ignore and report list

## Tests

- patch submit updates answers and advances
- invalid patch returns error and does not advance
- submit-all completes form and returns complete status

## Acceptance criteria

- end-to-end: render_text -> submit_patch -> next -> completes

