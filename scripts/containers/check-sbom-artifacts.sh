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
import os
import sys

root = Path(sys.argv[1])
manifest_root = root / "artifacts/containers"
if not manifest_root.exists():
    if "CI" in os.environ:
        print("sbom artifacts: missing artifacts/containers", file=sys.stderr)
        raise SystemExit(1)
    print("sbom artifacts: SKIP (no artifacts/containers)")
    raise SystemExit(0)

errors = []
seen = 0
for mf in sorted(manifest_root.glob("*.json")):
    if mf.name in {"summary.json", "report.json"}:
        continue
    try:
        data = json.loads(mf.read_text(encoding="utf-8"))
    except Exception:
        continue
    if data.get("status") != "ok":
        continue
    seen += 1
    sbom = str(data.get("sbom_path", "")).strip()
    if not sbom:
        errors.append(f"{mf.name}: missing sbom_path")
        continue
    if not Path(sbom).exists():
        errors.append(f"{mf.name}: sbom_path does not exist: {sbom}")

if errors:
    print("sbom artifacts: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print(f"sbom artifacts: OK ({seen} manifests)")
PY
