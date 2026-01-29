# QA-PR-10 — Adaptive Card 1.3 renderer (“card” transport)

> **Goal:** Adaptive Card 1.3 renderer (“card” transport)

## Summary

Add Adaptive Card v1.3 renderer (“card mode”).

This renderer is used for interactive wizard flows and provider setup.

## Card constraints (must follow)

- Adaptive Card `version`: "1.3"
- Use only elements widely supported:
  - `TextBlock`, `Container`, `FactSet`
  - `Input.Text`, `Input.ChoiceSet`, `Input.Toggle`
  - `Action.Submit`, `Action.OpenUrl`

## Standard Action.Submit payload shapes

### Patch mode (one question)
```
{
  "qa": {
    "formId": "...",
    "mode": "patch",
    "questionId": "...",
    "field": "answer"  // where UI input is stored
  }
}
```

### Submit-all mode
```
{
  "qa": {
    "formId": "...",
    "mode": "all"
  }
}
```

Choose a consistent design: either embed answers directly in submit payload or rely on input ids.
Document clearly.

## UI design rules (MVP)

- Top: Title + optional progress
- Middle: Intro (optional)
- Current question input (single-question card for v1)
- Bottom: submit button "Next ➡️" and optional "Back" if enabled

## Tests

- snapshot test JSON of produced card
- validate required fields exist: type, version, body[], actions[]
- ensure no unsupported fields included

## Acceptance criteria

- `render_card` produces valid AC 1.3 JSON
- patch mode works with submit handler (next PR)

