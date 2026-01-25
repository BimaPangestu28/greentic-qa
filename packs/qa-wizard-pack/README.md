# qa-wizard-pack

This deployable automation pack runs a QA wizard flow that emits `qa.wizard.generated` artifacts for downstream tooling. Its manifest, flows, and dependencies live under `packs/qa-wizard-pack/` and were generated via the Greentic CLIs:

- `greentic-pack new qa-wizard-pack`
- `greentic-flow new` / `greentic-flow add-step`
- `greentic-pack build` / `greentic-pack doctor`

The `pack.yaml` defines the `qa_wizard` flow, links to the shared `qa.process` + `messaging` dependencies, and declares the event contract in this README. Re-run the CLI if the flow or manifest is altered.

## Flow overview

- `collect_metadata` uses `qa.process` to capture form identifiers, titles, descriptions, and the preferred output directory.
- `collect_question` captures a representative question (id, prompt, type, choices) that seeds the rendered FormSpec/flow artifacts.
- `emit_event` uses `messaging.emit` to publish a `qa.wizard.generated` event carrying the generated files + summary.

See `flows/qa_wizard.flow.yaml` for the exact node layout and routing logic.

## Generated event

Downstream systems consume the `qa.wizard.generated` event emitted by `emit_event`. The payload delivers the serialized metadata needed to produce a FormSpec bundle:

```json
{
  "dir_name": "<directory name selected by the wizard>",
  "summary_md": "<user-supplied summary>",
  "form": {
    "id": "...",
    "title": "...",
    "version": "...",
    "description": "...",
    "progress_policy": {
      "skip_answered": true,
      "autofill_defaults": false,
      "treat_default_as_answered": false
    }
  },
  "questions": [
    {
      "id": "...",
      "type": "...",
      "title": "...",
      "description": "...",
      "required": true,
      "default_value": "...",
      "secret": false
    }
  ]
}
```

Use `qa-cli generate --input <event>.json --out <dir>` (see `ci/scripts/smoke.sh`) to digest the event and emit the normalized `forms/`, `flows/`, `examples/`, and `schemas/` artifacts. The CLI respects `QA_WIZARD_OUTPUT_DIR` for dev-mode persistence and `QA_WIZARD_ALLOWED_ROOTS` to restrict where files can be written; add `--force` if you intentionally want to overwrite previous bundles.

## Local iteration

1. Build the pack via `greentic-pack build --in packs/qa-wizard-pack --dry-run`.
2. Update `flows/qa_wizard.flow.yaml` or the supporting flow assets.
3. Re-run the CLI to regenerate `pack.lock.json` (if needed) and verify the wizard still emits the expected event.

Do not hand-edit generated lockfiles; rerun `greentic-pack` whenever the manifest or flow changes.
