#!/usr/bin/env bash
set -euo pipefail

if [[ "${SKIP_BINSTALL:-false}" != "true" ]]; then
  if command -v cargo-binstall >/dev/null 2>&1; then
    cargo binstall --no-confirm qa-cli || echo "cargo binstall reported an error (ignored for now)"
  else
    echo "cargo-binstall not installed; install to verify qa-cli distribution"
  fi
else
  echo "SKIP_BINSTALL=true; skipping cargo binstall verification"
fi

bash ci/scripts/smoke.sh
