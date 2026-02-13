#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import json
import sys

root = Path(sys.argv[1])
golden_root = root / "assets" / "golden" / "toy-runs-v1"
errors: list[str] = []

for d in sorted(golden_root.glob("*")):
    if not d.is_dir():
        continue
    rel = d.relative_to(root)
    manifest = d / "manifest.json"
    metrics = d / "metrics.json"
    checksums = d / "artifact_checksums.json"
    for p in (manifest, metrics, checksums):
        if not p.exists():
            errors.append(f"{rel}: missing {p.name}")
    if not manifest.exists() or not metrics.exists() or not checksums.exists():
        continue

    try:
        m = json.loads(manifest.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"{rel}/manifest.json invalid JSON: {exc}")
        continue
    for key in ("schema_version", "profile_id", "domain", "generated_at", "inputs_root"):
        if key not in m:
            errors.append(f"{rel}/manifest.json missing key '{key}'")

    try:
        met = json.loads(metrics.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"{rel}/metrics.json invalid JSON: {exc}")
        continue
    for key in ("schema_version", "generated_at", "input_checksums"):
        if key not in met:
            errors.append(f"{rel}/metrics.json missing key '{key}'")
    if not isinstance(met.get("input_checksums"), dict):
        errors.append(f"{rel}/metrics.json input_checksums must be an object")

    try:
        c = json.loads(checksums.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"{rel}/artifact_checksums.json invalid JSON: {exc}")
        continue
    for key in ("schema_version", "profile_id", "generated_at", "artifacts"):
        if key not in c:
            errors.append(f"{rel}/artifact_checksums.json missing key '{key}'")
    artifacts = c.get("artifacts")
    if not isinstance(artifacts, dict):
        errors.append(f"{rel}/artifact_checksums.json artifacts must be an object")
    else:
        for k in ("manifest.json", "metrics.json", "report.html"):
            if k not in artifacts:
                errors.append(f"{rel}/artifact_checksums.json artifacts missing '{k}'")

if errors:
    print("golden-artifact-schema: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("golden-artifact-schema: OK")
PY
