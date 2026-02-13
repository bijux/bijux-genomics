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
LOCK_SNAPSHOT_FILE="$OUT_DIR/last_lock.sha256"

plan_only=0
only_tool=""
changed_only=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --plan) plan_only=1 ;;
    --only)
      only_tool="${2:-}"
      if [[ -z "$only_tool" ]]; then
        echo "--only requires <tool-id>" >&2
        exit 2
      fi
      shift
      ;;
    --changed) changed_only=1 ;;
    --help|-h)
      cat <<'USAGE'
Usage: scripts/containers/ensure-images.sh [--plan] [--only <tool-id>] [--changed]
USAGE
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
  shift
done

if [[ -n "$only_tool" && "$changed_only" == "1" ]]; then
  echo "--only and --changed are mutually exclusive" >&2
  exit 2
fi

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

selected_tools_json='[]'

if [[ -n "$only_tool" ]]; then
  selected_tools_json="[\"$only_tool\"]"
fi

if [[ "$changed_only" == "1" ]]; then
  prev_lock=""
  if [[ -f "$LOCK_SNAPSHOT_FILE" ]]; then
    prev_lock="$(tr -d '[:space:]' < "$LOCK_SNAPSHOT_FILE")"
  fi
  if [[ -z "$prev_lock" || "$prev_lock" == "$lock_sha" ]]; then
    selected_tools_json='[]'
  else
    selected_tools_json=$(python3 - "$ROOT_DIR" "$prev_lock" <<'PY'
import subprocess
import sys
from pathlib import Path

root = Path(sys.argv[1])
prev_lock = sys.argv[2]

def rev_for_lock(lock):
    p = subprocess.run(["git", "-C", str(root), "log", "--format=%H", "--all", "--", "configs/ci/registry/tool_registry_lock.sha256"],
                       capture_output=True, text=True, check=False)
    for rev in p.stdout.splitlines():
        q = subprocess.run(["git", "-C", str(root), "show", f"{rev}:configs/ci/registry/tool_registry_lock.sha256"],
                           capture_output=True, text=True, check=False)
        if q.returncode == 0 and q.stdout.strip() == lock:
            return rev
    return ""

rev = rev_for_lock(prev_lock)
if not rev:
    print("[]")
    raise SystemExit(0)

p = subprocess.run(["git", "-C", str(root), "diff", "--name-only", f"{rev}..HEAD", "--", "containers/apptainer", "containers/docker"],
                   capture_output=True, text=True, check=False)

tools = set()
for line in p.stdout.splitlines():
    line = line.strip()
    if not line:
        continue
    name = Path(line).name
    if name.startswith("Dockerfile."):
        tools.add(name.split("Dockerfile.", 1)[1])
    elif name.endswith(".def"):
        tools.add(name[:-4])

items = ",".join(f'"{t}"' for t in sorted(tools))
print(f"[{items}]")
PY
)
  fi
fi

if [[ -n "$prev_sha" && "$prev_sha" == "$combined_sha" && "$plan_only" -eq 0 && -z "$only_tool" && "$changed_only" -eq 0 ]]; then
  echo "ensure-images: skip rebuild (config+lock unchanged)"
  cat > "$REPORT" <<JSON
{
  "schema_version": "bijux.containers.ensure_images.v3",
  "action": "skip",
  "reason": "unchanged",
  "images_toml": "${IMAGES_TOML#"$ROOT_DIR/"}",
  "tool_registry_lock": "configs/ci/registry/tool_registry_lock.sha256",
  "images_sha": "$images_sha",
  "lock_sha": "$lock_sha",
  "combined_sha": "$combined_sha",
  "selected_tools": $selected_tools_json
}
JSON
  exit 0
fi

if [[ "$plan_only" -eq 1 ]]; then
  cat > "$REPORT" <<JSON
{
  "schema_version": "bijux.containers.ensure_images.v3",
  "action": "plan",
  "reason": "requested",
  "images_toml": "${IMAGES_TOML#"$ROOT_DIR/"}",
  "tool_registry_lock": "configs/ci/registry/tool_registry_lock.sha256",
  "images_sha": "$images_sha",
  "lock_sha": "$lock_sha",
  "combined_sha": "$combined_sha",
  "selected_tools": $selected_tools_json
}
JSON
  echo "plan: wrote $REPORT"
  exit 0
fi

build_one_apptainer_tool() {
  local tool="$1"
  if [[ -f "$ROOT_DIR/containers/apptainer/bijux/${tool}.def" ]]; then
    ./scripts/run.sh containers build-apptainer-all \
      --defs-dir containers/apptainer/bijux \
      --build-one "$ROOT_DIR/containers/apptainer/bijux/${tool}.def" \
      --vm-out "${HOME}/apptainer-build" \
      --copy-back "$ROOT_DIR/artifacts/containers/apptainer"
  elif [[ -f "$ROOT_DIR/containers/apptainer/non-bijux/${tool}.def" ]]; then
    ./scripts/run.sh containers build-apptainer-all \
      --defs-dir containers/apptainer/non-bijux \
      --build-one "$ROOT_DIR/containers/apptainer/non-bijux/${tool}.def" \
      --vm-out "${HOME}/apptainer-build" \
      --copy-back "$ROOT_DIR/artifacts/containers/apptainer"
  else
    echo "ensure-images: no apptainer def found for tool: $tool" >&2
    return 1
  fi
}

if [[ -n "$only_tool" ]]; then
  build_one_apptainer_tool "$only_tool"
elif [[ "$changed_only" == "1" ]]; then
  mapfile -t changed_tools < <(printf '%s\n' "$selected_tools_json" | tr -d '[]"' | tr ',' '\n' | sed '/^$/d')
  if [[ "${#changed_tools[@]}" -eq 0 ]]; then
    echo "ensure-images: --changed found no tool def deltas since last lock snapshot"
  else
    for tool in "${changed_tools[@]}"; do
      build_one_apptainer_tool "$tool"
    done
  fi
else
  ./scripts/run.sh containers build-apptainer-all --defs-dir containers/apptainer --vm-out "${HOME}/apptainer-build" --copy-back "$ROOT_DIR/artifacts/containers/apptainer"
fi

cat > "$REPORT" <<JSON
{
  "schema_version": "bijux.containers.ensure_images.v3",
  "action": "rebuild",
  "reason": "requested_or_changed",
  "images_toml": "${IMAGES_TOML#"$ROOT_DIR/"}",
  "tool_registry_lock": "configs/ci/registry/tool_registry_lock.sha256",
  "images_sha": "$images_sha",
  "lock_sha": "$lock_sha",
  "combined_sha": "$combined_sha",
  "selected_tools": $selected_tools_json
}
JSON

printf '%s\n' "$lock_sha" > "$LOCK_SNAPSHOT_FILE"
echo "ensure-images: wrote $REPORT"
