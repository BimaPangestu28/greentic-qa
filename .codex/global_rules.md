# Global Rules

These guidelines augment the repository-wide rules described in the PR bundle. They are intentionally conservative so later PRs can rely on a predictable foundation.

## Visibility policies
- Introduce `visibility_on_missing: "visible" | "hidden" | "error"` when evaluating `visible_if`. Default to `"visible"` for interactive/card flows so no question is silently skipped. Automated/validation-only workflows should default to `"error"` so missing context fails loudly.
- Document the chosen policy per PR and ensure it surfaces in `qa-spec` tests (fixtures should cover both `"visible"` and `"error"` behaviors when dependencies are unavailable).

## Secret handling
- Keywords:
  - `secret_access_denied`: raised when a secrets policy explicitly forbids a key.
  - `secret_host_unavailable`: raised when policy allows a key but the host lacks the secrets-store capability.
- Do not embed secret values in error messages. At most include the key name (and only if strictly necessary).
- Reuse `greentic-secrets` + the standard secrets-store WIT interface; do not invent new secret APIs. Gate reads/writes via `SecretsPolicy` (enable/read/write flags plus allow/deny globs).

## Wizard pack persistence
- Default persistence mode remains *event only* (`qa.wizard.generated`). Do not add local writers unless explicitly enabled.
- Dev-mode writer requires:
  - `QA_WIZARD_OUTPUT_DIR` under an allowed root (GC-managed workspace/temp root).
  - Files emitted into `<QA_WIZARD_OUTPUT_DIR>/<dir_name>/…`, never arbitrary paths.
  - Optional `QA_WIZARD_ALLOWED_ROOTS` to harden host config.
- Document the dev-writer mode and its safety checks in both the workflow docs and pack README.

## CLI/tooling mandates
- Always scaffold components/packs/flows via the approved CLIs:
  - `greentic-component new component-qa` → implement logic inside generated crate.
  - `greentic-flow new` / `greentic-flow add-step` → tweak flows only via CLI updates.
  - `greentic-pack new qa-wizard-pack`, then `greentic-pack build`/`doctor`; never unzip or manually edit packed artifacts.
- Only apply hand edits that the CLI cannot generate; document them and keep them minimal.
- Do not bypass `doctor`/`build` steps. If a change requires regeneration, rerun the CLI.

## Foundational reuse
- Align QA models with existing shared packages:
  - Prefer `greentic-types` / `greentic-interfaces` for platform-wide models.
  - QA-specific models may live in `qa-spec`, but keep names/serializations compatible with interface crates.
  - Share validation/templating logic where it already exists instead of duplicating.
- When referencing `questions` concepts, extend the shared definitions rather than inventing parallel type systems.

## Hard rules (apply to every PR)
✅ Allowed:
- Scaffolding via the approved CLIs plus small, focused edits when the CLI output needs tuning.
- Reusing existing shared models (`greentic-types`, `greentic-interfaces`, etc.) and existing secrets policies (`greentic-secrets`).

❌ Not allowed:
- Handcrafting component manifests, flow YAML/JSON, or pack metadata from scratch.
- Unzipping or patching pack internals instead of using the CLI.
- Skipping CLI `doctor`/`build` steps or manually editing generated binaries before committing.
