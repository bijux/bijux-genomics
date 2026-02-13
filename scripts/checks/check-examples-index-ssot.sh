#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys
try:
    import yaml  # type: ignore
except Exception:
    yaml = None

root = Path(sys.argv[1])
idx = root / "examples" / "index.yaml"
if not idx.exists():
    print("examples index ssoT: missing examples/index.yaml", file=sys.stderr)
    raise SystemExit(1)

if yaml is None:
    # minimal parse fallback
    text = idx.read_text(encoding="utf-8")
    rows = []
    cur = {}
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("- id:"):
            if cur:
                rows.append(cur)
            cur = {"id": s.split(":", 1)[1].strip()}
        elif s.startswith("path:"):
            cur["path"] = s.split(":", 1)[1].strip()
    if cur:
        rows.append(cur)
else:
    data = yaml.safe_load(idx.read_text(encoding="utf-8"))
    rows = data.get("examples", [])

errors = []
for row in rows:
    ex_id = str(row.get("id", "")).strip()
    path = str(row.get("path", "")).strip()
    outs = row.get("expected_outputs", [])
    ex = root / path
    if not ex_id:
        errors.append("examples/index.yaml entry missing id")
        continue
    if not path:
        errors.append(f"examples/index.yaml entry '{ex_id}' missing path")
        continue
    if not ex.exists():
        errors.append(f"examples/index.yaml entry '{ex_id}' path missing: {path}")
        continue
    if not (ex / "example.toml").exists():
        errors.append(f"{path}: missing manifest example.toml")
    if not (ex / "golden" / "plan.json").exists():
        errors.append(f"{path}: missing golden/plan.json")
    if not (ex / "golden" / "explain.json").exists():
        errors.append(f"{path}: missing golden/explain.json")
    if not (ex / "golden" / "report.json").exists():
        errors.append(f"{path}: missing golden/report.json")
    if isinstance(outs, list):
        need = {"plan.json", "explain.json", "report.json"}
        if not need.issubset(set(str(x) for x in outs)):
            errors.append(f"{path}: expected_outputs must include plan.json/explain.json/report.json")

if errors:
    print("examples index ssoT: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("examples index ssoT: OK")
PY
