#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

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
