#!/usr/bin/env bash
set -euo pipefail

SMOKE_OUTPUT_DIR=${SMOKE_OUTPUT_DIR:-smoke-output}
SMOKE_INPUT=${SMOKE_INPUT:-ci/fixtures/sample_form_generation.json}

rm -rf "$SMOKE_OUTPUT_DIR"
mkdir -p "$SMOKE_OUTPUT_DIR"

cargo run -p qa-cli -- --help
cargo run -p qa-cli -- generate --input "$SMOKE_INPUT" --out "$SMOKE_OUTPUT_DIR"

BUNDLE_NAME=$(python3 - <<'PY'
import json, sys
with open(sys.argv[1]) as fh:
    data = json.load(fh)
    print(data.get("dir_name") or "")
PY
  "$SMOKE_INPUT")

if [[ -z "$BUNDLE_NAME" ]]; then
  echo "Unable to read bundle name from $SMOKE_INPUT"
  exit 1
fi

BUNDLE_PATH="$SMOKE_OUTPUT_DIR/$BUNDLE_NAME"

if [[ ! -d "$BUNDLE_PATH/forms" || ! -d "$BUNDLE_PATH/flows" || ! -d "$BUNDLE_PATH/examples" || ! -d "$BUNDLE_PATH/schemas" ]]; then
  echo "Generated bundle missing expected directories under $BUNDLE_PATH"
  exit 1
fi

echo "Smoke artifacts generated under $BUNDLE_PATH"
