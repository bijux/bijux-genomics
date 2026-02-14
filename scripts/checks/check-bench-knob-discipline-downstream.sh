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
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
bench_knobs = root / "configs/bench/knobs.toml"
if not bench_knobs.exists():
    print("bench downstream discipline: missing configs/bench/knobs.toml", file=sys.stderr)
    raise SystemExit(1)

mini_bench = [
    root / "examples/vcf/downstream-vcf-full-mini/bench-suite.toml",
    root / "examples/vcf/downstream-demography-mini/bench-suite.toml",
    root / "examples/vcf/imputation-mini/bench-suite.toml",
]
errors = []
for p in mini_bench:
    if not p.exists():
        errors.append(f"missing bench suite: {p}")
        continue
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    sid = str(data.get("suite_id", "")).strip()
    stages = data.get("stages", [])
    if not sid:
        errors.append(f"{p}: missing suite_id")
    if not isinstance(stages, list) or not stages:
        errors.append(f"{p}: stages must be non-empty list")
    if not all(str(s).startswith("vcf.") for s in stages):
        errors.append(f"{p}: all stages must be vcf.*")

knobs = tomllib.loads(bench_knobs.read_text(encoding="utf-8"))
defaults = knobs.get("defaults", {})
for k in ("warmup_policy", "repetitions", "capture_cpu", "capture_memory", "capture_io"):
    if k not in defaults:
        errors.append(f"configs/bench/knobs.toml defaults missing {k}")

if errors:
    print("bench downstream discipline: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("bench downstream discipline: OK")
PY
