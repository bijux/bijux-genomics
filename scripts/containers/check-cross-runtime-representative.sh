#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

docker_dir="${1:-$ROOT_DIR/artifacts/containers/docker-arm64}"
apptainer_dir="${2:-$ROOT_DIR/artifacts/containers/apptainer}"

python3 - "$docker_dir" "$apptainer_dir" <<'PY'
from pathlib import Path
import json
import re
import sys

def load(path: Path):
    out = {}
    for p in sorted(path.glob("*.json")):
        if p.name in {"summary.json", "report.json", "lock.json"}:
            continue
        try:
            d = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        t = str(d.get("tool", "")).strip()
        if t:
            out[t] = d
    return out


def norm(s: str) -> str:
    return re.sub(r"\s+", " ", s.strip().lower())

d = Path(sys.argv[1])
a = Path(sys.argv[2])
if not d.exists() or not a.exists():
    if __import__("os").environ.get("CI"):
        print("cross-runtime representative: missing runtime dirs", file=sys.stderr)
        raise SystemExit(1)
    print("cross-runtime representative: SKIP (missing runtime dirs)")
    raise SystemExit(0)

dm = load(d)
am = load(a)
shared = sorted(set(dm) & set(am))
if len(shared) < 5:
    if __import__("os").environ.get("CI"):
        print(f"cross-runtime representative: need >=5 shared tools, found {len(shared)}", file=sys.stderr)
        raise SystemExit(1)
    print(f"cross-runtime representative: SKIP (<5 shared tools, found {len(shared)})")
    raise SystemExit(0)

# Deterministic representative set: first 5 alphabetical
rep = shared[:5]
errors = []
for tool in rep:
    dd = dm[tool]
    aa = am[tool]
    if dd.get("status") != "ok" or aa.get("status") != "ok":
        errors.append(f"{tool}: non-ok status docker={dd.get('status')} apptainer={aa.get('status')}")
        continue
    dv = norm(str(dd.get("version_output", "")))
    av = norm(str(aa.get("version_output", "")))
    if not dv or not av or dv != av:
        errors.append(f"{tool}: version_output mismatch docker='{dv}' apptainer='{av}'")

if errors:
    print("cross-runtime representative: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print(f"cross-runtime representative: OK ({', '.join(rep)})")
PY
