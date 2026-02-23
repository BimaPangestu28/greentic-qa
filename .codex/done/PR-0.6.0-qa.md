# PR: greentic-qa – 0.6.0 Wizard Upgrades (Conditions, Repeatables, Computed, Validation)

**Base branch:** `release/0.6.0`  
Urgent fixes go to `master` and are cherry-picked.

## Goal

Upgrade greentic-qa to serve as the **uniform question/answer runtime** for:
- `greentic-component wizard`
- `greentic-pack wizard`

Scope is “minimum viable dynamic setup” without turning QA into a programming language.

## Must-have features (0.6.0)

### 1) Conditional visibility

Add per-question and per-section optional condition:

- `visible_if: Expr`

Where `Expr` supports:
- boolean ops: `and`, `or`, `not`
- comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- existence: `is_set("path")`
- path reads: `answer("q1")` or `answer("section.q")`

Keep it deterministic; no external calls.

### 2) Repeatable groups (safe “loop”)

Add a question type like:

- `List<Record>` with `min_items`, `max_items`, and per-item fields.

Example: “Add channels” -> list of `{ name: string, id: string }`.

### 3) Computed fields (pure derivations)

Add computed questions/fields:

- `computed: Expr`
- not directly user-editable (or allow override with flag)

Used for:
- slug generation
- default subject strings
- derived NATS subjects, etc.

### 4) Validation

- per-field validation:
  - required
  - min/max
  - regex
  - enum
- cross-field validation rules:
  - “if A then B required”
  - “either X or Y must be set”

Output should keep errors user-friendly.

### 5) CBOR answers output

Ensure wizard can output answers as:
- canonical CBOR bytes
- and (optionally) JSON for debugging

Provide a stable “answers object” layout.

## Explicitly deferred (0.6.1+)

- Imperative tool steps (loop/condition blocks as flow nodes)
- WASM tools inside QA
- external lookups (tenant/provider enumeration) unless already trivial

Add an extension hook interface placeholder if helpful, but keep it unused.

## CLI UX cleanup (requested)

Reduce internal noise in CLI output:
- don’t print “internal states” by default
- show hints for booleans: `(yes/no)` / `(y/n)` / `(true/false)`
- show required marker consistently
- on validation error: show expected type + examples

Add `--verbose` to show the old detailed debug state.

## Tests

Add fixtures + tests for:
- visibility_if toggling
- repeatable group add/remove
- computed field evaluation
- cross-field validation errors
- CBOR output stable decode

## Acceptance criteria

- existing specs still work (backwards compatible where possible)
- wizard outputs deterministic CBOR suitable to embed in manifests/examples
