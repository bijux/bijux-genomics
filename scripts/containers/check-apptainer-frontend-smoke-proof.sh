#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

PROOF_ROOT="${1:-$ROOT_DIR/artifacts/containers/hpc/frontend-smoke}"

python3 - "$ROOT_DIR" "$PROOF_ROOT" <<'PY'
from pathlib import Path
import json
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
proof_root = Path(sys.argv[2])
summary = proof_root / "summary.json"
if not summary.exists():
    if "CI" in __import__("os").environ:
        print(f"frontend smoke proof: missing {summary}", file=sys.stderr)
        raise SystemExit(1)
    print("frontend smoke proof: SKIP (no summary)")
    raise SystemExit(0)

data = json.loads(summary.read_text(encoding="utf-8"))
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
apptainer_tools = set(p.stem for p in (root / "containers/apptainer/bijux").glob("*.def")) | set(p.stem for p in (root / "containers/apptainer/non-bijux").glob("*.def"))
items = {str(i.get("tool", "")).strip(): i for i in data.get("items", [])}
errors = []

for t in sorted(apptainer_tools):
    row = items.get(t)
    if not row:
        errors.append(f"{t}: missing smoke proof row")
        continue
    if str(row.get("status", "")) != "ok":
        errors.append(f"{t}: smoke status not ok")
        continue
    # version match
    out = str(row.get("normalized_version_output", "") or row.get("version_output", "")).strip().lower()
    expected = str((versions.get(t) or {}).get("version", "")).strip().lower()
    if expected and expected not in out:
        errors.append(f"{t}: version output does not include expected version {expected}")
    # exit code checks
    for key in ("help_actual_exit_code", "minimal_actual_exit_code", "negative_actual_exit_code"):
        if row.get(key) is None:
            errors.append(f"{t}: missing {key}")
    # runtime network / home / writes policy flags from manifest
    if row.get("network_runtime_detected") is True:
        errors.append(f"{t}: runtime network access detected")
    if row.get("home_write_detected") is True:
        errors.append(f"{t}: write to HOME detected")
    if row.get("write_policy_ok") is not True:
        errors.append(f"{t}: write_policy_ok is false")

if errors:
    print("frontend smoke proof: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print(f"frontend smoke proof: OK ({len(apptainer_tools)} tools)")
PY
