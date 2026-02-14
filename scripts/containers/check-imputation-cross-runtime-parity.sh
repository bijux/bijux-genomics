#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

DOCKER_SUMMARY="${1:-$ROOT_DIR/artifacts/containers/docker-arm64/summary.json}"
APPTAINER_SUMMARY="${2:-$ROOT_DIR/artifacts/containers/apptainer/summary.json}"

if [[ ! -f "$DOCKER_SUMMARY" || ! -f "$APPTAINER_SUMMARY" ]]; then
  if [[ -n "${CI:-}" ]]; then
    echo "imputation cross-runtime parity: missing summary files docker='$DOCKER_SUMMARY' apptainer='$APPTAINER_SUMMARY'" >&2
    exit 1
  fi
  echo "imputation cross-runtime parity: SKIP (missing local summary files)"
  exit 0
fi

python3 - "$DOCKER_SUMMARY" "$APPTAINER_SUMMARY" <<'PY'
import json
import re
import sys
from pathlib import Path

tools = ["glimpse", "impute5", "minimac4", "shapeit5", "beagle", "eagle", "bcftools", "plink2"]
docker = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
appt = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
errors = []

def rows(summary):
    out = {}
    for row in summary.get("items", []):
        out[str(row.get("tool", "")).strip()] = row
    return out

def norm(s: str) -> str:
    return re.sub(r"[^a-z0-9.]+", " ", s.lower()).strip()

d = rows(docker)
a = rows(appt)
for t in tools:
    dr = d.get(t)
    ar = a.get(t)
    if dr is None or ar is None:
        errors.append(f"{t}: missing from one runtime summary")
        continue
    dv = norm(str(dr.get("version_output", "")))
    av = norm(str(ar.get("version_output", "")))
    if not dv or not av:
        errors.append(f"{t}: empty version output for parity check")
        continue
    if t not in dv or t not in av:
        errors.append(f"{t}: version outputs do not contain expected tool token")
        continue
    if str(dr.get("declared_version", "")).strip() not in ("", "unknown", "planned", "latest-pinned"):
        declared = str(dr.get("declared_version")).lower()
        if declared not in dv or declared not in av:
            errors.append(f"{t}: declared_version `{declared}` not present in both runtime outputs")

if errors:
    print("imputation cross-runtime parity: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("imputation cross-runtime parity: OK")
PY
