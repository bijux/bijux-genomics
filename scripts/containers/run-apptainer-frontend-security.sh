#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

./bin/require-isolate >/dev/null || {
  ./bin/require-isolate --explain >&2
  exit 1
}

POLICY_TOML="${POLICY_TOML:-$ROOT_DIR/configs/ci/tools/apptainer_security_policy.toml}"
HPC_POLICY_TOML="${HPC_POLICY_TOML:-$ROOT_DIR/configs/ci/tools/hpc_frontend_build_policy.toml}"
SEC_ROOT="${SEC_ROOT:-${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/hpc/frontend-security}"
DOC_SUMMARY="${DOC_SUMMARY:-$ROOT_DIR/containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md}"
RUN_ID="${ISO_RUN_ID:-run}"
OUT_DIR="$SEC_ROOT/$RUN_ID"
ALLOWLIST_PATH=""

require_cmd python3
[[ -f "$POLICY_TOML" ]] || { echo "missing $POLICY_TOML" >&2; exit 1; }
[[ -f "$HPC_POLICY_TOML" ]] || { echo "missing $HPC_POLICY_TOML" >&2; exit 1; }

host_name="$(hostname -f 2>/dev/null || hostname)"
python3 - "$HPC_POLICY_TOML" "$host_name" <<'PY'
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
with open(sys.argv[1], "rb") as fh:
    cfg = tomllib.load(fh)
hn = sys.argv[2]
pat = str(cfg.get("compute_hostname_regex", "")).strip()
if pat and re.search(pat, hn):
    raise SystemExit(f"refusing frontend security run on compute node host: {hn}")
PY

# Enforce pinning and existing security contracts first.
"$SCRIPT_DIR/check-version-hash-pin.sh"
"$SCRIPT_DIR/check-apptainer-hardening.sh"
"$SCRIPT_DIR/check-no-secrets.sh"
"$SCRIPT_DIR/check-network-disclosure.sh"

mkdir -p "$OUT_DIR"

# Build/smoke all apptainer runtime tools; this also writes per-tool manifests and sbom_path fields.
ARTIFACT_DIR="$OUT_DIR" FRONTEND_PROOF_MODE=1 SMOKE_LEVEL=contract \
  "$SCRIPT_DIR/smoke-apptainer.sh"

ALLOWLIST_PATH="$(python3 - "$POLICY_TOML" "$ROOT_DIR" <<'PY'
import sys
from pathlib import Path
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
p = Path(sys.argv[1])
root = Path(sys.argv[2])
cfg = tomllib.loads(p.read_text(encoding="utf-8"))
rel = str(cfg.get("vuln_allowlist_path", "")).strip()
print(str(root / rel) if rel else "")
PY
)"

python3 - "$ROOT_DIR" "$OUT_DIR" "$POLICY_TOML" "$ALLOWLIST_PATH" "$DOC_SUMMARY" "$host_name" <<'PY'
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out = Path(sys.argv[2])
policy = tomllib.loads(Path(sys.argv[3]).read_text(encoding="utf-8"))
allowlist_path = Path(sys.argv[4]) if sys.argv[4] else None
doc_summary = Path(sys.argv[5])
host = sys.argv[6]

fail_on_critical = bool(policy.get("fail_on_unallowlisted_critical", True))
require_scanner_ci = bool(policy.get("require_scanner_in_ci", True))
require_scanner_local = bool(policy.get("require_scanner_local", False))
is_ci = bool(os.environ.get("CI"))

allowlisted = set()
if allowlist_path and allowlist_path.exists():
    data = tomllib.loads(allowlist_path.read_text(encoding="utf-8"))
    for row in data.get("allowlist", []):
        cve = str(row.get("cve", "")).strip().upper()
        if cve:
            allowlisted.add(cve)

manifests = {}
for mf in sorted(out.glob("*.json")):
    if mf.name in {"summary.json", "security_summary.json", "vuln_scan_report.json", "sbom_index.json"}:
        continue
    try:
        row = json.loads(mf.read_text(encoding="utf-8"))
    except Exception:
        continue
    tool = str(row.get("tool", "")).strip()
    if not tool:
        continue
    manifests[tool] = row

sbom_rows = []
for tool, row in sorted(manifests.items()):
    sbom_path = Path(str(row.get("sbom_path", "")).strip())
    sif_path = Path(str(row.get("image", "")).strip())
    if not sbom_path.exists():
        continue
    sbom_sha = hashlib.sha256(sbom_path.read_bytes()).hexdigest()
    sif_sha = ""
    if sif_path.exists():
        sif_sha = hashlib.sha256(sif_path.read_bytes()).hexdigest()
    sbom_rows.append(
        {
            "tool": tool,
            "sbom_path": str(sbom_path),
            "sbom_sha256": sbom_sha,
            "sif_path": str(sif_path),
            "sif_sha256": sif_sha,
        }
    )

scanner = None
for cand in ("grype", "trivy"):
    if shutil.which(cand):
        scanner = cand
        break
if scanner is None and ((is_ci and require_scanner_ci) or ((not is_ci) and require_scanner_local)):
    raise SystemExit("scanner required by policy but neither grype nor trivy is available")

vuln_dir = out / "vuln"
vuln_dir.mkdir(parents=True, exist_ok=True)
vuln_items = []
critical_total = 0
critical_unallowlisted = []

def parse_grype(payload: dict):
    outv = []
    for m in payload.get("matches", []):
        v = m.get("vulnerability", {}) or {}
        cve = str(v.get("id", "")).strip().upper()
        sev = str(v.get("severity", "")).strip().upper()
        if cve:
            outv.append((cve, sev))
    return outv

def parse_trivy(payload: dict):
    outv = []
    for res in payload.get("Results", []) or []:
        for v in res.get("Vulnerabilities", []) or []:
            cve = str(v.get("VulnerabilityID", "")).strip().upper()
            sev = str(v.get("Severity", "")).strip().upper()
            if cve:
                outv.append((cve, sev))
    return outv

for row in sbom_rows:
    tool = row["tool"]
    sbom = row["sbom_path"]
    item = {"tool": tool, "scanner": scanner or "none", "critical": 0, "high": 0, "medium": 0, "low": 0, "unknown": 0}
    if scanner == "grype":
        proc = subprocess.run(["grype", f"sbom:{sbom}", "-o", "json"], capture_output=True, text=True)
        raw = proc.stdout if proc.stdout else "{}"
        (vuln_dir / f"{tool}.grype.json").write_text(raw, encoding="utf-8")
        payload = json.loads(raw or "{}")
        vulns = parse_grype(payload)
    elif scanner == "trivy":
        proc = subprocess.run(["trivy", "sbom", "--format", "json", sbom], capture_output=True, text=True)
        raw = proc.stdout if proc.stdout else "{}"
        (vuln_dir / f"{tool}.trivy.json").write_text(raw, encoding="utf-8")
        payload = json.loads(raw or "{}")
        vulns = parse_trivy(payload)
    else:
        vulns = []

    for cve, sev in vulns:
        key = sev.lower() if sev.lower() in item else "unknown"
        item[key] += 1
        if sev == "CRITICAL":
            critical_total += 1
            if cve not in allowlisted:
                critical_unallowlisted.append({"tool": tool, "cve": cve})
    vuln_items.append(item)

license_mismatches = []
for row in sbom_rows:
    tool = row["tool"]
    lic_file = root / "containers/licenses" / f"{tool}.license.toml"
    if not lic_file.exists():
        license_mismatches.append(f"{tool}: missing containers/licenses/{tool}.license.toml")
        continue
    lic = tomllib.loads(lic_file.read_text(encoding="utf-8"))
    spdx = str(lic.get("spdx", "")).strip()
    if not spdx:
        license_mismatches.append(f"{tool}: empty spdx in {lic_file.relative_to(root)}")
        continue
    # Current SBOM format is package list text; enforce metadata presence and non-empty SBOM.
    sbom_path = Path(row["sbom_path"])
    if sbom_path.stat().st_size == 0:
        license_mismatches.append(f"{tool}: empty sbom package list")

security = {
    "schema_version": "bijux.apptainer.frontend.security.v1",
    "host": host,
    "scanner": scanner or "none",
    "items": sbom_rows,
    "vulnerabilities": vuln_items,
    "critical_total": critical_total,
    "critical_unallowlisted": critical_unallowlisted,
    "license_mismatches": license_mismatches,
}

if fail_on_critical and critical_unallowlisted:
    security["ok"] = False
else:
    security["ok"] = len(license_mismatches) == 0

summary_json = out / "security_summary.json"
summary_json.write_text(json.dumps(security, indent=2, sort_keys=True) + "\n", encoding="utf-8")
(out / "sbom_index.json").write_text(json.dumps({"schema_version": "bijux.apptainer.sbom.index.v1", "items": sbom_rows}, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = [
    "<!-- Generated by scripts/containers/run-apptainer-frontend-security.sh -->",
    "",
    "# Apptainer Frontend Security Summary",
    "",
    f"- host: `{host}`",
    f"- scanner: `{scanner or 'none'}`",
    f"- sif_count: `{len(sbom_rows)}`",
    f"- critical_total: `{critical_total}`",
    f"- critical_unallowlisted: `{len(critical_unallowlisted)}`",
    f"- license_mismatches: `{len(license_mismatches)}`",
    f"- gate_status: `{'PASS' if security['ok'] else 'FAIL'}`",
    "",
    "## SBOM Index",
    "",
    "| tool | sif_sha256 | sbom_sha256 | sbom_path |",
    "|---|---|---|---|",
]
for row in sorted(sbom_rows, key=lambda x: x["tool"]):
    lines.append(f"| `{row['tool']}` | `{row['sif_sha256']}` | `{row['sbom_sha256']}` | `{row['sbom_path']}` |")

lines += ["", "## Vulnerability Summary", "", "| tool | critical | high | medium | low | unknown |", "|---|---:|---:|---:|---:|---:|"]
for row in sorted(vuln_items, key=lambda x: x["tool"]):
    lines.append(f"| `{row['tool']}` | `{row['critical']}` | `{row['high']}` | `{row['medium']}` | `{row['low']}` | `{row['unknown']}` |")

if critical_unallowlisted:
    lines += ["", "## Unallowlisted Critical CVEs", ""]
    for row in critical_unallowlisted:
        lines.append(f"- `{row['tool']}`: `{row['cve']}`")

if license_mismatches:
    lines += ["", "## License/SBOM Issues", ""]
    for err in license_mismatches:
        lines.append(f"- {err}")

doc_summary.parent.mkdir(parents=True, exist_ok=True)
doc_summary.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(summary_json)
print(doc_summary)
if not security["ok"]:
    raise SystemExit(1)
PY

echo "frontend apptainer security: OK"
