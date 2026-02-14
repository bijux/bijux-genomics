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
    print("frontend observability proof: SKIP (no frontend-mini-e2e artifacts)")
    raise SystemExit(0)
runs = sorted([p for p in base.iterdir() if p.is_dir()])
if not runs:
    print("frontend observability proof: SKIP (no run dirs)")
    raise SystemExit(0)
run = runs[-1]
summary = json.loads((run / "summary.json").read_text(encoding="utf-8"))
errors = []

for row in summary.get("examples", []):
    art = Path(str(row.get("artifact_dir", "")))
    meta = art / "frontend_run_meta.json"
    log = art / "logs.txt"
    if not meta.exists() or not log.exists():
        errors.append(f"{art}: missing observability files")
        continue
    m = json.loads(meta.read_text(encoding="utf-8"))
    for k in (
        "tool_versions_ref",
        "container_lock_sha256",
        "domain_hash_sha256",
        "config_hash_sha256",
        "start_utc",
        "end_utc",
        "exit_code",
    ):
        if str(m.get(k, "")).strip() == "":
            errors.append(f"{art}: meta missing {k}")
    lt = log.read_text(encoding="utf-8")
    for marker in ("example_id=", "corpus_id=", "step1=", "step2=", "step3=", "step4="):
        if marker not in lt:
            errors.append(f"{art}: logs missing {marker}")

if errors:
    print("frontend observability proof: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print(f"frontend observability proof: OK ({run.name})")
PY
