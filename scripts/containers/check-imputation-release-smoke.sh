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
    echo "imputation release smoke: missing summary files docker='$DOCKER_SUMMARY' apptainer='$APPTAINER_SUMMARY'" >&2
    exit 1
  fi
  echo "imputation release smoke: SKIP (missing local summary files)"
  exit 0
fi

python3 - "$DOCKER_SUMMARY" "$APPTAINER_SUMMARY" <<'PY'
import json
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

for runtime, data in (("docker", rows(docker)), ("apptainer", rows(appt))):
    for t in tools:
        row = data.get(t)
        if not row:
            errors.append(f"{runtime}:{t}: missing summary row")
            continue
        if str(row.get("status", "")) != "ok":
            errors.append(f"{runtime}:{t}: status is not ok")
        paths = row.get("smoke_output_paths", {})
        for key in ("version", "help"):
            p = str(paths.get(key, "")).strip()
            if not p:
                errors.append(f"{runtime}:{t}: missing smoke_output_paths.{key}")
                continue
            if not Path(p).exists():
                errors.append(f"{runtime}:{t}: missing output file {p}")
        if not str(row.get("version_output", "")).strip():
            errors.append(f"{runtime}:{t}: empty version_output")
        if not str(row.get("resolved_image_digest", "")).strip():
            errors.append(f"{runtime}:{t}: missing resolved_image_digest")

if errors:
    print("imputation release smoke: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("imputation release smoke: OK")
PY
