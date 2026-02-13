#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SBOM_ROOT="${1:-${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/sbom}"
OUT="${2:-${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/vuln_scan_report.json}"
TOOLKIT="${TOOLKIT:-}"
PROMOTED_ONLY="${PROMOTED_ONLY:-1}"
ensure_artifacts_dir "$(dirname "$OUT")"

python3 - "$ROOT_DIR" "$SBOM_ROOT" "$OUT" "$TOOLKIT" "$PROMOTED_ONLY" <<'PY'
from pathlib import Path
import json
import shutil
import subprocess
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
sbom_root = Path(sys.argv[2])
out = Path(sys.argv[3])
toolkit = str(sys.argv[4]).strip()
promoted_only = str(sys.argv[5]).strip() in {"1", "true", "yes"}
scanner = None
for cand in ("grype", "trivy"):
    if shutil.which(cand):
        scanner = cand
        break

allowed_tools = set()
if promoted_only:
    lock = root / "containers/versions/lock.json"
    if lock.exists():
        lock_data = json.loads(lock.read_text(encoding="utf-8"))
        allowed_tools |= {
            str(item.get("tool", "")).strip()
            for item in lock_data.get("items", [])
            if str(item.get("status", "")).strip() == "production"
        }
if toolkit:
    bundles_path = root / "configs/ci/tools/toolkit_bundles.toml"
    bundles = tomllib.loads(bundles_path.read_text(encoding="utf-8")) if bundles_path.exists() else {}
    tools = bundles.get("bundles", {}).get(toolkit, {}).get("tools", [])
    allowed_tools &= set(str(t).strip() for t in tools) if allowed_tools else set(str(t).strip() for t in tools)

rows = []
for p in sorted(sbom_root.rglob("*.packages.txt")):
    tool = p.parent.name
    if allowed_tools and tool not in allowed_tools:
        continue
    row = {"sbom": str(p), "scanner": scanner or "none", "status": "not_scanned", "summary": ""}
    if scanner == "grype":
        proc = subprocess.run(["grype", f"sbom:{p}", "-o", "json"], capture_output=True, text=True)
        row["status"] = "ok" if proc.returncode == 0 else "error"
        row["summary"] = proc.stdout[:2000] if proc.stdout else proc.stderr[:500]
    elif scanner == "trivy":
        proc = subprocess.run(["trivy", "sbom", "--format", "json", str(p)], capture_output=True, text=True)
        row["status"] = "ok" if proc.returncode == 0 else "error"
        row["summary"] = proc.stdout[:2000] if proc.stdout else proc.stderr[:500]
    row["tool"] = tool
    rows.append(row)

payload = {
    "schema_version": "bijux.container.vuln_hook.v1",
    "scanner": scanner or "none",
    "toolkit": toolkit or "all",
    "promoted_only": promoted_only,
    "items": rows,
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"wrote {out}")
PY
