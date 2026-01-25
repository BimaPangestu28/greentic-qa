# QA-PR-15 — Publishing + binstall + smoke tests (real)

> **Goal:** Publishing + binstall + smoke tests (real)

## Summary

Make publishing and installation verification real.

- Ensure crates publish cleanly in dependency order.
- Provide a `smoke` script that CI can run after install.
- Ensure `qa-cli` is installable via `cargo binstall` on all target OS runners.

## Deliverables

- update `publish.yml` to publish:
  1) `qa-spec`
  2) `component-qa` (only if you publish wasm crate; optional)
  3) `qa-cli`

- Add `ci/scripts/smoke.sh`:
  - `qa-cli --help`
  - `qa-cli generate ...` to create output directory
  - validate schema and example file exist

- Add `ci/scripts/binstall_verify.sh`:
  - installs via binstall
  - runs smoke

## Notes

- If `component-qa` is not a crates.io crate (because it’s a component artifact), publish only `qa-spec` and `qa-cli`.
- Document the release artifact strategy in README.

## Acceptance criteria

- publish workflow succeeds on a dry-run branch (or manual test)
- binstall verification passes for linux/windows/macos matrices
- smoke tests provide meaningful signal

