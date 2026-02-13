#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'USAGE'
Usage: scripts/checks/check-benchmark-integrity-policy.sh
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
errs = []

bench_script = (root / "scripts/tooling/benchmarks.sh").read_text(encoding="utf-8")
if "./bin/require-isolate" not in bench_script:
    errs.append("scripts/tooling/benchmarks.sh must enforce require-isolate")
if "/benchmarks/" not in bench_script:
    errs.append("scripts/tooling/benchmarks.sh must default outputs under benchmarks/")
if "containers/smoke" not in bench_script:
    errs.append("scripts/tooling/benchmarks.sh must guard against smoke/benchmark log mixing")

knobs_path = root / "configs/bench/knobs.toml"
if not knobs_path.exists():
    errs.append("configs/bench/knobs.toml missing")
else:
    cfg = tomllib.loads(knobs_path.read_text(encoding="utf-8"))
    var = cfg.get("variance")
    if not isinstance(var, dict):
        errs.append("configs/bench/knobs.toml missing [variance] section")
    else:
        for k in ("runtime_relative_max", "memory_relative_max", "report_structure_match"):
            if k not in var:
                errs.append(f"configs/bench/knobs.toml [variance] missing '{k}'")

var_doc = root / "docs/30-operations/BENCHMARK_VARIANCE.md"
if not var_doc.exists():
    errs.append("docs/30-operations/BENCHMARK_VARIANCE.md missing")
else:
    txt = var_doc.read_text(encoding="utf-8")
    for phrase in ("runtime relative variance", "memory relative variance", "report.html"):
        if phrase.lower() not in txt.lower():
            errs.append(f"docs/30-operations/BENCHMARK_VARIANCE.md missing phrase '{phrase}'")

if errs:
    print("benchmark-integrity-policy: FAILED", file=sys.stderr)
    for e in errs:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("benchmark-integrity-policy: OK")
PY
