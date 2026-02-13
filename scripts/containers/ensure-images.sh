#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

IMAGES_TOML_PRIMARY="$ROOT_DIR/configs/ci/images.toml"
IMAGES_TOML_FALLBACK="$ROOT_DIR/configs/ci/tools/images.toml"
LOCK_SHA_FILE="$ROOT_DIR/configs/ci/registry/tool_registry_lock.sha256"
OUT_DIR="$ROOT_DIR/artifacts/containers/ensure-images"
REPORT="$OUT_DIR/report.json"
plan_only=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --plan) plan_only=1 ;;
    --help|-h)
      cat <<'EOF'
Usage: scripts/containers/ensure-images.sh [--plan]
EOF
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
  shift
done

if [[ -f "$IMAGES_TOML_PRIMARY" ]]; then
  IMAGES_TOML="$IMAGES_TOML_PRIMARY"
else
  IMAGES_TOML="$IMAGES_TOML_FALLBACK"
fi
[[ -f "$IMAGES_TOML" ]] || { echo "missing $IMAGES_TOML_PRIMARY and $IMAGES_TOML_FALLBACK" >&2; exit 1; }
[[ -f "$LOCK_SHA_FILE" ]] || { echo "missing $LOCK_SHA_FILE" >&2; exit 1; }

ensure_artifacts_dir "$OUT_DIR"
mkdir -p "$OUT_DIR"

images_sha=$(shasum -a 256 "$IMAGES_TOML" | awk '{print $1}')
lock_sha=$(tr -d '[:space:]' < "$LOCK_SHA_FILE")
combined_sha=$(printf '%s\n%s\n' "$images_sha" "$lock_sha" | shasum -a 256 | awk '{print $1}')

prev_sha=""
if [[ -f "$REPORT" ]]; then
  prev_sha=$(rg -No '"combined_sha"\s*:\s*"([a-f0-9]+)"' "$REPORT" | sed -E 's/.*"([a-f0-9]+)"/\1/' || true)
fi

if [[ -n "$prev_sha" && "$prev_sha" == "$combined_sha" ]]; then
  echo "ensure-images: skip rebuild (config+lock unchanged)"
  cat > "$REPORT" <<JSON
{
  "schema_version": "bijux.containers.ensure_images.v2",
  "action": "skip",
  "reason": "unchanged",
  "images_toml": "${IMAGES_TOML#"$ROOT_DIR/"}",
  "tool_registry_lock": "configs/ci/registry/tool_registry_lock.sha256",
  "images_sha": "$images_sha",
  "lock_sha": "$lock_sha",
  "combined_sha": "$combined_sha"
}
JSON
  if [[ "$plan_only" == "1" ]]; then
    echo "plan: action=skip reason=unchanged images_sha=$images_sha lock_sha=$lock_sha"
  fi
  exit 0
fi

echo "ensure-images: rebuild required (config+lock changed)"
if [[ "$plan_only" == "1" ]]; then
  cat > "$REPORT" <<JSON
{
  "schema_version": "bijux.containers.ensure_images.v2",
  "action": "plan-rebuild",
  "reason": "changed",
  "images_toml": "${IMAGES_TOML#"$ROOT_DIR/"}",
  "tool_registry_lock": "configs/ci/registry/tool_registry_lock.sha256",
  "images_sha": "$images_sha",
  "lock_sha": "$lock_sha",
  "combined_sha": "$combined_sha"
}
JSON
  echo "plan: action=rebuild reason=changed images_sha=$images_sha lock_sha=$lock_sha"
  exit 0
fi

./scripts/run.sh containers build-apptainer-all --defs-dir containers/apptainer --vm-out "${HOME}/apptainer-build" --copy-back "$ROOT_DIR/artifacts/containers/apptainer"

cat > "$REPORT" <<JSON
{
  "schema_version": "bijux.containers.ensure_images.v2",
  "action": "rebuild",
  "reason": "changed",
  "images_toml": "${IMAGES_TOML#"$ROOT_DIR/"}",
  "tool_registry_lock": "configs/ci/registry/tool_registry_lock.sha256",
  "images_sha": "$images_sha",
  "lock_sha": "$lock_sha",
  "combined_sha": "$combined_sha"
}
JSON

echo "ensure-images: wrote $REPORT"
