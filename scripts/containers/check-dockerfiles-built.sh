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
in_ci = bool(__import__("os").environ.get("CI"))
summary_path = root / "artifacts/containers/summary.json"
expected_tools = sorted(p.name.split("Dockerfile.", 1)[1] for p in (root / "containers/docker/arm64").glob("Dockerfile.*"))

if not in_ci:
    print("dockerfiles built check: SKIP (CI-only gate)")
    raise SystemExit(0)

if not summary_path.exists():
    print("dockerfiles built check: missing artifacts/containers/summary.json", file=sys.stderr)
    raise SystemExit(1)

summary = json.loads(summary_path.read_text(encoding="utf-8"))
rows = {}
for item in summary.get("items", []):
    tool = str(item.get("tool", "")).strip()
    runtime = str(item.get("runtime", "")).strip()
    if runtime != "docker-arm64" or not tool:
        continue
    rows[tool] = item

errors = []
for tool in expected_tools:
    row = rows.get(tool)
    if not row:
        errors.append(f"{tool}: missing docker-arm64 summary row")
        continue
    if str(row.get("status", "")).strip() != "ok":
        errors.append(f"{tool}: build/smoke status is not ok")
        continue
    manifest_path = Path(str(row.get("manifest", "")))
    if not manifest_path.exists():
        errors.append(f"{tool}: manifest missing at {manifest_path}")
        continue
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception:
        errors.append(f"{tool}: manifest is invalid JSON ({manifest_path})")
        continue
    digest = str(manifest.get("resolved_image_digest", "")).strip()
    if not digest:
        errors.append(f"{tool}: missing resolved_image_digest in manifest")

if errors:
    print("dockerfiles built check: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("dockerfiles built check: OK")
PY
