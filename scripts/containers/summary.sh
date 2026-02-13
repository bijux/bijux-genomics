#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
MANIFEST_DIR="${MANIFEST_DIR:-$ROOT_DIR/artifacts/container}"
json_out=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --json)
      if [[ -n "${2:-}" && "${2:-}" != --* ]]; then
        json_out="$2"
        shift
      else
        json_out="__default__"
      fi
      ;;
    --help|-h)
      cat <<'EOF'
Usage: scripts/containers/summary.sh [--json <output-path>]
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

if [ ! -d "$MANIFEST_DIR" ]; then
  echo "no manifests found: $MANIFEST_DIR" >&2
  exit 2
fi

printf "tool\truntime\tresult\tlog\n"
for f in "$MANIFEST_DIR"/*.json; do
  [ -e "$f" ] || continue
  tool=$(awk -F'"' '/"tool"/ {print $4; exit}' "$f")
  runtime=$(awk -F'"' '/"runtime"/ {print $4; exit}' "$f")
  status=$(awk -F'"' '/"status"/ {print $4; exit}' "$f")
  log="$MANIFEST_DIR/logs/${runtime}/${tool}.log"
  printf "%s\t%s\t%s\t%s\n" "$tool" "$runtime" "$status" "$log"
done | sort

if [[ -n "$json_out" ]]; then
  if [[ "$json_out" == "__default__" ]]; then
    json_out="$ROOT_DIR/artifacts/container/summary/summary.json"
  fi
  if [[ -d "$json_out" ]]; then
    rm -rf "$json_out"
  fi
  ensure_artifacts_dir "$(dirname "$json_out")"
  python3 - <<'PY' "$MANIFEST_DIR" "$json_out"
import glob
import json
import os
import sys

manifest_dir = sys.argv[1]
out_path = sys.argv[2]
rows = []
for path in sorted(glob.glob(os.path.join(manifest_dir, "*.json"))):
    try:
      with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    except Exception:
      continue
    tool = data.get("tool", "")
    runtime = data.get("runtime", "")
    status = data.get("status", "")
    log = os.path.join(manifest_dir, "logs", runtime, f"{tool}.log") if tool and runtime else ""
    rows.append({"tool": tool, "runtime": runtime, "status": status, "log": log, "manifest": path})

os.makedirs(os.path.dirname(out_path), exist_ok=True)
with open(out_path, "w", encoding="utf-8") as f:
    json.dump({"schema_version": "bijux.container.summary.v1", "items": rows}, f, indent=2)
print(out_path)
PY
fi
