# QA-PR-08 — Secrets integration (default deny) + templating support (gated)

> **Goal:** Secrets integration (default deny) + templating support (gated)

## Summary

Add secrets support with **default deny**.

- Extend context with `secrets` (read is gated)
- Allow writing secrets via store mapping (write is gated)
- Provide allow/deny patterns

## Deliverables

### qa-spec
- `SecretsPolicy`:
  - `enabled: bool` default false
  - `read_enabled: bool` default false
  - `write_enabled: bool` default false
  - `allow: Vec<String>` glob-like patterns
  - `deny: Vec<String>`

- `secrets.rs`:
  - pattern matching helper
  - function to determine which keys may be loaded

### component-qa
- define an integration point for secrets-store host calls:
  - `get_secret(key)` and `put_secret(key,value)` **only** if allowed
- if running without secrets host support, return a clear error explaining host capability missing

## Security rules

- Never include secret values in error messages.
- When templating references a denied secret:
  - return deterministic error code like `secret_access_denied`
- When rendering (later PR), secrets should not be printed unless explicitly allowed.

## Tests

- deny-by-default: references fail
- allowlist works
- deny overrides allow
- write blocked if write_disabled

## Acceptance criteria

- secrets cannot be accessed unless explicitly enabled + allowlisted
- test suite covers both allowed and denied behavior

