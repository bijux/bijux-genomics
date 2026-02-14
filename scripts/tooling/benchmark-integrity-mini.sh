#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'USAGE'
Usage: scripts/tooling/benchmark-integrity-mini.sh [--sample-id <id>] [--r1 <fastq>] [--out <dir>]
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

./bin/require-isolate >/dev/null || {
  ./bin/require-isolate --explain >&2
  exit 1
}

sample_id="mini_bench"
r1="$ROOT_DIR/assets/toy/core-v1/fastq/reads_1.fastq"
base_out="${ISO_ROOT:-$ROOT_DIR/artifacts}/benchmarks/integrity-mini/${ISO_RUN_ID:-manual}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --sample-id) sample_id="${2:-}"; shift 2 ;;
    --r1) r1="${2:-}"; shift 2 ;;
    --out) base_out="${2:-}"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -n "$sample_id" ]] || { echo "empty --sample-id" >&2; exit 2; }
[[ -f "$r1" ]] || { echo "missing r1 fastq: $r1" >&2; exit 1; }

mkdir -p "$base_out"
run_a="$base_out/run_a"
run_b="$base_out/run_b"
mkdir -p "$run_a" "$run_b"

OUT_DIR="$run_a" SAMPLE_ID="$sample_id" R1="$r1" STAGE="validate" "$ROOT_DIR/scripts/tooling/benchmarks.sh" fastq-stage
OUT_DIR="$run_b" SAMPLE_ID="$sample_id" R1="$r1" STAGE="validate" "$ROOT_DIR/scripts/tooling/benchmarks.sh" fastq-stage

python3 - "$ROOT_DIR" "$run_a" "$run_b" <<'PY'
from pathlib import Path
import json
import math
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
run_a = Path(sys.argv[2])
run_b = Path(sys.argv[3])
errs = []

knobs = tomllib.loads((root / "configs/bench/knobs.toml").read_text(encoding="utf-8"))
variance = knobs.get("variance", {})
runtime_rel_max = float(variance.get("runtime_relative_max", 0.20))
memory_rel_max = float(variance.get("memory_relative_max", 0.25))

def load_json(p: Path):
    return json.loads(p.read_text(encoding="utf-8"))

def find_first(base: Path, name: str):
    vals = sorted(base.rglob(name))
    return vals[0] if vals else None

for p in (run_a, run_b):
    if "containers/smoke" in str(p):
        errs.append(f"{p}: benchmark output path overlaps smoke")

m_a = find_first(run_a, "metrics.json")
m_b = find_first(run_b, "metrics.json")
t_a = find_first(run_a, "telemetry.jsonl")
t_b = find_first(run_b, "telemetry.jsonl")
h_a = find_first(run_a, "report.html")
h_b = find_first(run_b, "report.html")

for tag, p in (("run_a", m_a), ("run_b", m_b), ("run_a", t_a), ("run_b", t_b), ("run_a", h_a), ("run_b", h_b)):
    if p is None:
        errs.append(f"{tag}: missing required artifact ({'metrics.json/telemetry.jsonl/report.html'})")

float_pat = re.compile(r"^-?\d+\.\d+$")

def walk(obj, fn):
    if isinstance(obj, dict):
        for v in obj.values():
            walk(v, fn)
    elif isinstance(obj, list):
        for v in obj:
            walk(v, fn)
    else:
        fn(obj)

runtime_values = []
memory_values = []

for tag, mp in (("run_a", m_a), ("run_b", m_b)):
    if mp is None:
        continue
    data = load_json(mp)
    def check_leaf(v):
        if isinstance(v, float):
            s = f"{v:.12f}".rstrip("0").rstrip(".")
            if "." in s and len(s.split(".", 1)[1]) > 6:
                errs.append(f"{tag}: excessive float precision in metrics ({v})")
    walk(data, check_leaf)
    txt = mp.read_text(encoding="utf-8")
    mem_match = re.findall(r'"memory_mb"\s*:\s*([0-9]+(?:\.[0-9]+)?)', txt)
    rt_match = re.findall(r'"(?:runtime_s|runtime_ms|duration_ms)"\s*:\s*([0-9]+(?:\.[0-9]+)?)', txt)
    if mem_match:
        memory_values.append(float(mem_match[0]))
    if rt_match:
        runtime_values.append(float(rt_match[0]))

for tag, tp in (("run_a", t_a), ("run_b", t_b)):
    if tp is None:
        continue
    by_stage = {}
    for i, line in enumerate(tp.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        row = json.loads(line)
        stage = str(row.get("stage_id", "")).strip()
        trace = str(row.get("trace_id", "")).strip()
        if not stage or not trace:
            errs.append(f"{tag}:{i}: missing stage_id/trace_id")
            continue
        if stage in by_stage and by_stage[stage] != trace:
            errs.append(f"{tag}:{i}: trace_id drift within stage {stage}")
        by_stage[stage] = trace
        if re.search(r"/Users/|/home/|\\btmp/", line):
            errs.append(f"{tag}:{i}: telemetry leaks host path")

def normalize_html(p: Path):
    text = p.read_text(encoding="utf-8")
    text = re.sub(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", "<TS>", text)
    text = re.sub(r"/Users/[^\"'< ]+", "<PATH>", text)
    text = re.sub(r"/home/[^\"'< ]+", "<PATH>", text)
    text = re.sub(r"run[_-]id[:=][^\"'< ]+", "run_id=<RUN>", text, flags=re.IGNORECASE)
    return text

if h_a and h_b:
    if normalize_html(h_a) != normalize_html(h_b):
        errs.append("report.html normalized structure differs across consecutive mini benchmark runs")

def rel_diff(a: float, b: float):
    d = max(abs(a), abs(b), 1e-9)
    return abs(a - b) / d

if len(runtime_values) == 2:
    d = rel_diff(runtime_values[0], runtime_values[1])
    if d > runtime_rel_max:
        errs.append(f"runtime variance {d:.4f} exceeds threshold {runtime_rel_max:.4f}")
if len(memory_values) == 2:
    d = rel_diff(memory_values[0], memory_values[1])
    if d > memory_rel_max:
        errs.append(f"memory variance {d:.4f} exceeds threshold {memory_rel_max:.4f}")

summary = {
    "schema_version": "bijux.benchmark.integrity.frontend-mini.v1",
    "run_a": str(run_a),
    "run_b": str(run_b),
    "runtime_relative_max": runtime_rel_max,
    "memory_relative_max": memory_rel_max,
    "runtime_values": runtime_values,
    "memory_values": memory_values,
    "ok": not errs,
    "errors": errs,
}
out = run_b.parent / "integrity_summary.json"
out.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(out)
if errs:
    print("benchmark integrity mini: FAILED", file=sys.stderr)
    for e in errs:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("benchmark integrity mini: OK")
PY
