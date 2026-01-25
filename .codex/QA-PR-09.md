# QA-PR-09 — Renderers: text + JSON UI

> **Goal:** Renderers: text + JSON UI

## Summary

Add renderers:
- `render_text`
- `render_json_ui`

These renderers are transport-agnostic and used by CLI and debugging.

## Output requirements

Both renderers must include:
- `form_id`
- `status` (`need_input|complete|error`)
- `next_question_id` (if any)
- `progress` (answered/total)
- `help`/`description` where available

`render_json_ui` must provide machine-friendly structure:
- questions list with:
  - id, type, title, description, required, default, current_value?, visible
- schemaRef if helpful (or embed the schema)

## Tests

- snapshot tests for render outputs (stable)
- verify progress counts correct

## Acceptance criteria

- CLI can call these renderers to show step-by-step prompts
- renderer respects visibility and policies

