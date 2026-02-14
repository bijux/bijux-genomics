#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
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
strict_promoted = bool(os.environ.get("CI")) or os.environ.get("REQUIRE_PROMOTED_SBOM") == "1"
lock_path = root / "containers/versions/lock.json"
lock = json.loads(lock_path.read_text(encoding="utf-8")) if lock_path.exists() else {"items": []}
promoted = {str(i.get("tool", "")).strip() for i in lock.get("items", []) if str(i.get("status", "")).strip() == "production"}

manifests = {}
for mf in sorted(manifest_root.glob("*.json")):
    if mf.name in {"summary.json", "report.json"}:
        continue
    try:
        data = json.loads(mf.read_text(encoding="utf-8"))
    except Exception:
        continue
    tool = str(data.get("tool", "")).strip()
    if not tool:
        continue
    manifests.setdefault(tool, []).append((mf, data))

seen = 0
tools_to_check = sorted(promoted) if strict_promoted else sorted(set(manifests.keys()) & promoted) or sorted(manifests.keys())
for tool in tools_to_check:
    rows = manifests.get(tool, [])
    if not rows:
        errors.append(f"{tool}: missing smoke/build manifest under artifacts/containers/")
        continue
    ok_rows = [(mf, d) for (mf, d) in rows if d.get("status") == "ok"]
    if not ok_rows:
        errors.append(f"{tool}: has manifests but no successful status=ok result")
        continue
    for mf, data in ok_rows:
        seen += 1
        sbom = str(data.get("sbom_path", "")).strip()
        smoke_log = str(data.get("smoke_log_path", "")).strip()
        smoke_log_sha = str(data.get("smoke_log_checksum_path", "")).strip()
        if not sbom:
            errors.append(f"{mf.name}: missing sbom_path")
            continue
        if not Path(sbom).exists():
            errors.append(f"{mf.name}: sbom_path does not exist: {sbom}")
        if not smoke_log or not Path(smoke_log).exists():
            errors.append(f"{mf.name}: missing smoke_log_path or file not found: {smoke_log}")
        if not smoke_log_sha or not Path(smoke_log_sha).exists():
            errors.append(f"{mf.name}: missing smoke_log_checksum_path or file not found: {smoke_log_sha}")

if errors:
    print("sbom artifacts: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print(f"sbom artifacts: OK ({seen} manifests)")
PY
