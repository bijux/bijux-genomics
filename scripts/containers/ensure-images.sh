#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

IMAGES_TOML="$ROOT_DIR/configs/ci/tools/images.toml"
LOCK_SHA_FILE="$ROOT_DIR/configs/ci/registry/tool_registry_lock.sha256"
OUT_DIR="$ROOT_DIR/artifacts/container/ensure-images"
MANIFEST="$OUT_DIR/manifest.json"

[[ -f "$IMAGES_TOML" ]] || { echo "missing $IMAGES_TOML" >&2; exit 1; }
[[ -f "$LOCK_SHA_FILE" ]] || { echo "missing $LOCK_SHA_FILE" >&2; exit 1; }

ensure_artifacts_dir "$OUT_DIR"
mkdir -p "$OUT_DIR"

images_sha=$(shasum -a 256 "$IMAGES_TOML" | awk '{print $1}')
lock_sha=$(tr -d '[:space:]' < "$LOCK_SHA_FILE")
combined_sha=$(printf '%s\n%s\n' "$images_sha" "$lock_sha" | shasum -a 256 | awk '{print $1}')

prev_sha=""
if [[ -f "$MANIFEST" ]]; then
  prev_sha=$(rg -No '"combined_sha"\s*:\s*"([a-f0-9]+)"' "$MANIFEST" | sed -E 's/.*"([a-f0-9]+)"/\1/' || true)
fi

if [[ -n "$prev_sha" && "$prev_sha" == "$combined_sha" ]]; then
  echo "ensure-images: config+lock unchanged, skipping rebuild"
  exit 0
fi

echo "ensure-images: change detected, rebuilding container images"
./scripts/run.sh containers build-apptainer-all --defs-dir containers/apptainer --vm-out "${HOME}/apptainer-build" --copy-back "$ROOT_DIR/artifacts/container/apptainer"

cat > "$MANIFEST" <<JSON
{
  "schema_version": "bijux.containers.ensure_images.v1",
  "images_toml": "configs/ci/tools/images.toml",
  "tool_registry_lock": "configs/ci/registry/tool_registry_lock.sha256",
  "images_sha": "$images_sha",
  "lock_sha": "$lock_sha",
  "combined_sha": "$combined_sha"
}
JSON

echo "ensure-images: wrote $MANIFEST"
