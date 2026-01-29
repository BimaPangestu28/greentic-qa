#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check || { echo "cargo fmt --all -- --check failed" >&2; exit 1; }
cargo clippy --workspace --all-targets -- -D warnings || { echo "cargo clippy --workspace --all-targets -- -D warnings failed" >&2; exit 1; }
cargo test --workspace || { echo "cargo test --workspace failed" >&2; exit 1; }

crates=(
  crates/qa-spec
  crates/component-qa
  crates/qa-cli
)

for crate in "${crates[@]}"; do
  echo "Local dry-run publish for $crate"
  cargo package \
    --manifest-path "$crate/Cargo.toml" \
    --locked \
    --allow-dirty
  cargo publish \
    --manifest-path "$crate/Cargo.toml" \
    --dry-run \
    --locked \
    --allow-dirty
done
