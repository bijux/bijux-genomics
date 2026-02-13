#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SBOM_ROOT="${1:-${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/sbom}"
OUT="${2:-${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/vuln_scan_report.json}"
ensure_artifacts_dir "$(dirname "$OUT")"

python3 - "$SBOM_ROOT" "$OUT" <<'PY'
from pathlib import Path
import json
import shutil
import subprocess
import sys

sbom_root = Path(sys.argv[1])
out = Path(sys.argv[2])
scanner = None
for cand in ("grype", "trivy"):
    if shutil.which(cand):
        scanner = cand
        break

rows = []
for p in sorted(sbom_root.rglob("*.packages.txt")):
    row = {"sbom": str(p), "scanner": scanner or "none", "status": "not_scanned", "summary": ""}
    if scanner == "grype":
        proc = subprocess.run(["grype", f"sbom:{p}", "-o", "json"], capture_output=True, text=True)
        row["status"] = "ok" if proc.returncode == 0 else "error"
        row["summary"] = proc.stdout[:2000] if proc.stdout else proc.stderr[:500]
    elif scanner == "trivy":
        proc = subprocess.run(["trivy", "sbom", "--format", "json", str(p)], capture_output=True, text=True)
        row["status"] = "ok" if proc.returncode == 0 else "error"
        row["summary"] = proc.stdout[:2000] if proc.stdout else proc.stderr[:500]
    rows.append(row)

payload = {"schema_version": "bijux.container.vuln_hook.v1", "scanner": scanner or "none", "items": rows}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"wrote {out}")
PY
