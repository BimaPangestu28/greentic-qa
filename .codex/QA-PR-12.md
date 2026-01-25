# QA-PR-12 — qa-cli: text-only wizard that generates QA specs

> **Goal:** qa-cli: text-only wizard that generates QA specs

## Summary

Implement `qa-cli` as a text-only wizard that generates QA specs and outputs the canonical directory:

```
my-provider-setup/
  forms/card.setup.form.json
  flows/card.setup.qaflow.json
  examples/card.setup.answers.example.json
  schemas/card.setup.answers.schema.json
  README.md
```

MVP: generate the linear FormSpec + derived artifacts.
Optionally generate a trivial QAFlowSpec that wraps the FormSpec (entry step message -> questions -> end) for future use.

## CLI commands

- `qa-cli new` (interactive prompts in terminal)
  - prompts for: id/title/version, intro text, policies
  - add questions loop
  - choose storage mapping defaults (config keys, secrets)
- `qa-cli generate --input wizard.answers.json --out dir` (non-interactive)
  - for CI use or repeatable generation

- `qa-cli validate --spec forms/... --answers answers.json`
  - uses qa-spec validation

## Implementation details

- No external UI deps.
- Use simple stdin prompt loop; keep output deterministic.
- Include a "summary" at the end.

## Tests

- golden test: feed scripted stdin (or use non-interactive mode)
- assert generated files exist and match schema expectations

## Acceptance criteria

- `qa-cli new` can create a working spec directory
- schema and example files match the FormSpec

