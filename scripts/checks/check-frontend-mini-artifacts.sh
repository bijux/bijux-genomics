#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

BASE="${BASE:-${ISO_ROOT:-$ROOT_DIR/artifacts}/hpc/frontend-mini-e2e}"

python3 - "$BASE" <<'PY'
from pathlib import Path
import json
import sys

base = Path(sys.argv[1])
if not base.exists():
    print("frontend mini artifacts: SKIP (no frontend-mini-e2e artifacts)")
    raise SystemExit(0)

runs = sorted([p for p in base.iterdir() if p.is_dir()])
if not runs:
    print("frontend mini artifacts: SKIP (no run dirs)")
    raise SystemExit(0)

run = runs[-1]
summary = run / "summary.json"
if not summary.exists():
    print(f"frontend mini artifacts: missing {summary}", file=sys.stderr)
    raise SystemExit(1)
payload = json.loads(summary.read_text(encoding="utf-8"))
errors = []
for row in payload.get("examples", []):
    art = Path(str(row.get("artifact_dir", "")))
    if not art.exists():
        errors.append(f"missing artifact dir {art}")
        continue
    for f in ("plan.json", "explain.json", "report.json", "run_report.json", "metrics.json", "logs.txt", "frontend_run_meta.json"):
        if not (art / f).exists():
            errors.append(f"{art}: missing {f}")

if errors:
    print("frontend mini artifacts: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print(f"frontend mini artifacts: OK ({run.name})")
PY
