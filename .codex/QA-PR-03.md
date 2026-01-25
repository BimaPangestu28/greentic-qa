# QA-PR-03 — qa-spec crate: FormSpec + QAFlowSpec + AnswerSet (versioned)

> **Goal:** qa-spec crate: FormSpec + QAFlowSpec + AnswerSet (versioned)

## Summary

Implement `qa-spec` crate with versioned data models:
- `FormSpec` (linear questions)
- `QAFlowSpec` (graph steps for complex wizards)
- `AnswerSet`, `ProgressState`, errors

Add serde + schemars + test fixtures.

## API surface

### Types
- `FormSpec { id, title, version, intro?, presentation?, progress_policy?, secrets_policy?, questions: Vec<QuestionSpec> }`
- `QAFlowSpec { id, title, version, entry: StepId, steps: BTreeMap<StepId, StepSpec>, policies... }`
- `AnswerSet { form_id, spec_version, answers: serde_json::Value, meta: Meta }`
- `ProgressState { current_step: Option<StepId>, completed: bool, history?: Vec<...> }`
- `ValidationError { question_id?, path, message, code? }`

### QuestionSpec (minimum)
- `id`, `type`, `title`, `description?`, `required?`
- constraints: `pattern?`, `min?`, `max?`, `min_len?`, `max_len?`
- `enum?` for choices
- `default?` (string that may be templated later)
- `secret?`
- `visible_if?` (expression)

### StepSpec (QAFlowSpec)
- `message` step: `{ mode: text|json|card, template, next }`
- `question` step: `{ question_id, next }`
- `decision` step: `{ cases: [{ if, goto }], default_goto }`
- `action` step: (stub for now; implemented later)
- `end` step

## Files

- `crates/qa-spec/src/lib.rs` (re-export modules)
- `crates/qa-spec/src/spec/form.rs`
- `crates/qa-spec/src/spec/flow.rs`
- `crates/qa-spec/src/spec/question.rs`
- `crates/qa-spec/src/answers.rs`
- `crates/qa-spec/src/expr.rs` (expression model)
- `crates/qa-spec/tests/roundtrip.rs`
- `crates/qa-spec/tests/fixtures/*.json`

## Tests

- JSON roundtrip tests for each fixture:
  - parse -> serialize -> parse stable
- `schemars` schema generation tests:
  - ensure `FormSpec` schema compiles
  - ensure `QAFlowSpec` schema compiles

## Acceptance criteria

- `qa-spec` is usable by other crates.
- Fixtures cover at least:
  - linear simple form
  - graph flow with decision
- `cargo test -p qa-spec` passes.

