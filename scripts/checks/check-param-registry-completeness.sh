#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
param_files = [
    root / "configs/ci/params/param_registry.toml",
    root / "configs/ci/params/param_registry_vcf.toml",
    root / "configs/ci/params/param_registry_downstream.toml",
]
known_stage_ids: set[str] = set()
registry_params: dict[str, set[str]] = {}
for p in param_files:
    d = tomllib.loads(p.read_text(encoding="utf-8"))
    for row in d.get("entries", []) + d.get("params", []):
        sid = str(row.get("stage_id", "")).strip()
        if sid:
            known_stage_ids.add(sid)
            vals = row.get("params", [])
            if isinstance(vals, list):
                registry_params[sid] = {str(v).strip() for v in vals if str(v).strip()}

errors: list[str] = []
def extract_stage_parameters(text: str) -> list[str]:
    # Only parse names under an explicit `parameters:` block.
    m = re.search(r"^parameters:\s*\n((?:^[ \t].*\n?)*)", text, re.MULTILINE)
    if not m:
        return []
    block = m.group(1)
    return [
        p.strip()
        for p in re.findall(r'^\s*-\s+name:\s*"?(.*?)"?\s*$', block, re.MULTILINE)
        if p.strip()
    ]


for f in sorted(root.glob("domain/*/stages/*.yaml")):
    if f.name.startswith("_"):
        continue
    text = f.read_text(encoding="utf-8")
    sm = re.search(r'^status:\s*"?([^"\n]+)"?', text, re.MULTILINE)
    status = sm.group(1).strip() if sm else ""
    if status not in {"production", "supported"}:
        continue
    m = re.search(r'^stage_id:\s*"?([^"\n]+)"?', text, re.MULTILINE)
    if not m:
        errors.append(f"{f.relative_to(root)}: missing stage_id")
        continue
    sid = m.group(1).strip()
    if sid not in known_stage_ids:
        errors.append(f"{f.relative_to(root)}: stage_id '{sid}' missing from param_registry*.toml")
        continue

    # Enforce no hidden knobs: stage parameter names must exist in registry params list when declared.
    stage_params = extract_stage_parameters(text)
    if stage_params and sid.startswith("vcf."):
        reg = registry_params.get(sid, set())
        if not reg:
            errors.append(f"{f.relative_to(root)}: stage declares parameters but registry has no params list for '{sid}'")
        else:
            for p_name in stage_params:
                if p_name not in reg:
                    errors.append(f"{f.relative_to(root)}: parameter '{p_name}' missing in param_registry entry for '{sid}'")

if errors:
    print("param-registry-completeness: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("param-registry-completeness: OK")
PY
