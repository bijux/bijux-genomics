#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
Usage: scripts/checks/check-run-directory-layout.sh [scan_root]
Validates deterministic run directory layout and run artifact envelope keys.
Default scan_root: artifacts/
USAGE
  exit 0
fi

scan_root="${1:-$ROOT_DIR/artifacts}"
if [[ ! -d "$scan_root" ]]; then
  echo "check-run-directory-layout: SKIP ($scan_root not found)"
  exit 0
fi

fail=0
while IFS= read -r manifest; do
  run_dir="$(dirname "$manifest")"
  run_artifacts="$run_dir/run_artifacts"
  for rel in \
    "run_manifest.json" \
    "run_artifacts/telemetry/events.jsonl" \
    "run_artifacts/dashboard/facts.jsonl"; do
    if [[ ! -f "$run_dir/$rel" ]]; then
      echo "check-run-directory-layout: missing ${run_dir#$ROOT_DIR/}/$rel" >&2
      fail=1
    fi
  done
  if [[ -d "$run_artifacts" ]]; then
    while IFS= read -r env_file; do
      for rel in manifest_json metrics_json checksums provenance logs; do
        if ! python3 - "$env_file" "$rel" >/dev/null <<'PY'
import json,sys
p,k=sys.argv[1],sys.argv[2]
v=json.load(open(p,encoding="utf-8"))
if k not in v:
    raise SystemExit(1)
PY
        then
          echo "check-run-directory-layout: envelope missing key '$rel' in ${env_file#$ROOT_DIR/}" >&2
          fail=1
        fi
      done
    done < <(find "$run_artifacts" -type f -name 'run_artifact_envelope.json' | sort)
  fi
done < <(find "$scan_root" -type f -name 'run_manifest.json' | sort)

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi
echo "check-run-directory-layout: OK"
