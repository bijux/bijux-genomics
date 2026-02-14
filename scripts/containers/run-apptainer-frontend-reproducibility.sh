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

POLICY_TOML="${POLICY_TOML:-$ROOT_DIR/configs/ci/tools/apptainer_reproducibility_policy.toml}"
HPC_POLICY_TOML="${HPC_POLICY_TOML:-$ROOT_DIR/configs/ci/tools/hpc_frontend_build_policy.toml}"
CACHE_POLICY_TOML="${CACHE_POLICY_TOML:-$ROOT_DIR/configs/ci/tools/apptainer_cache_policy.toml}"
OUT_ROOT="${OUT_ROOT:-${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/hpc/frontend-reproducibility}"
DOC_REPORT="${DOC_REPORT:-$ROOT_DIR/containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md}"
SEED="${REPRO_SEED:-${ISO_RUN_ID:-frontend-repro}}"
SAMPLE_COUNT="${SAMPLE_COUNT:-}"
STRICT="${STRICT:-1}"

require_cmd apptainer
require_cmd python3
require_cmd shasum

[[ -f "$POLICY_TOML" ]] || { echo "missing $POLICY_TOML" >&2; exit 1; }
[[ -f "$HPC_POLICY_TOML" ]] || { echo "missing $HPC_POLICY_TOML" >&2; exit 1; }
[[ -f "$CACHE_POLICY_TOML" ]] || { echo "missing $CACHE_POLICY_TOML" >&2; exit 1; }

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
    raise SystemExit(f"refusing reproducibility run on compute node host: {hn}")
PY

# Frontend deterministic builds must use pinned versions only.
"$SCRIPT_DIR/check-version-hash-pin.sh"

default_count="$(python3 - "$POLICY_TOML" <<'PY'
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
with open(sys.argv[1], "rb") as fh:
    cfg = tomllib.load(fh)
print(int(cfg.get("tool_sample_count", 10)))
PY
)"
if [[ -z "$SAMPLE_COUNT" ]]; then
  SAMPLE_COUNT="$default_count"
fi

SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git -C "$ROOT_DIR" log -1 --format=%ct 2>/dev/null || echo 0)}"
export TZ="UTC" LC_ALL="C" LANG="C" SOURCE_DATE_EPOCH

APPTAINER_CACHEDIR="${APPTAINER_CACHEDIR:-$ISO_ROOT/cache/apptainer}"
APPTAINER_TMPDIR="${APPTAINER_TMPDIR:-$ISO_ROOT/tmp/apptainer}"
export APPTAINER_CACHEDIR APPTAINER_TMPDIR
mkdir -p "$APPTAINER_CACHEDIR" "$APPTAINER_TMPDIR" "$OUT_ROOT"

run_dir="$OUT_ROOT/${ISO_RUN_ID:-run}"
mkdir -p "$run_dir/builds" "$run_dir/logs"

defs_json="$run_dir/defs.json"
python3 - "$ROOT_DIR" "$defs_json" "$SEED" "$SAMPLE_COUNT" <<'PY'
import json
import random
import sys
from pathlib import Path

root = Path(sys.argv[1])
out = Path(sys.argv[2])
seed = sys.argv[3]
count = int(sys.argv[4])
defs = sorted((root / "containers" / "apptainer").glob("*/*.def"))
if not defs:
    raise SystemExit("no apptainer defs found")
if count > len(defs):
    count = len(defs)
rng = random.Random(seed)
sampled = rng.sample(defs, count)
payload = {
    "seed": seed,
    "tool_count": count,
    "defs": [{"tool": p.stem, "path": str(p)} for p in sampled],
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(out)
PY

cache_clean() {
  apptainer cache clean -f >/dev/null 2>&1 || true
  mkdir -p "$APPTAINER_CACHEDIR" "$APPTAINER_TMPDIR"
}

cache_purge() {
  rm -rf "$APPTAINER_CACHEDIR" "$APPTAINER_TMPDIR"
  mkdir -p "$APPTAINER_CACHEDIR" "$APPTAINER_TMPDIR"
}

build_one() {
  local tool="$1"
  local def_path="$2"
  local tag="$3"
  local out_sif="$run_dir/builds/${tool}.${tag}.sif"
  local out_log="$run_dir/logs/${tool}.${tag}.log"
  if apptainer build --force "$out_sif" "$def_path" >"$out_log" 2>&1; then
    :
  else
    echo "build failed: tool=$tool phase=$tag log=$out_log" >&2
    return 1
  fi
  shasum -a 256 "$out_sif" | awk '{print $1}'
}

summary_json="$run_dir/summary.json"
python3 - "$POLICY_TOML" "$summary_json" "$host_name" "$SEED" <<'PY'
import json
import sys
from pathlib import Path
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
with open(sys.argv[1], "rb") as fh:
    cfg = tomllib.load(fh)
out = Path(sys.argv[2])
payload = {
    "schema_version": "bijux.apptainer.frontend_reproducibility.v1",
    "host": sys.argv[3],
    "seed": sys.argv[4],
    "confidence_min": float(cfg.get("confidence_min", 1.0)),
    "require_all_tools_deterministic": bool(cfg.get("require_all_tools_deterministic", True)),
    "items": [],
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

while IFS=$'\t' read -r tool def_path; do
  [[ -n "$tool" ]] || continue

  h_base1="$(build_one "$tool" "$def_path" "baseline1")"
  h_base2="$(build_one "$tool" "$def_path" "baseline2")"

  cache_clean
  h_clean="$(build_one "$tool" "$def_path" "clean_cache")"

  cache_purge
  h_purge="$(build_one "$tool" "$def_path" "purge_cache")"

  python3 - "$summary_json" "$tool" "$def_path" "$h_base1" "$h_base2" "$h_clean" "$h_purge" <<'PY'
import json
import re
import sys
from pathlib import Path

summary = Path(sys.argv[1])
tool = sys.argv[2]
def_path = sys.argv[3]
h_base1, h_base2, h_clean, h_purge = sys.argv[4], sys.argv[5], sys.argv[6], sys.argv[7]

pair_equal = h_base1 == h_base2
clean_equal = h_base1 == h_clean
purge_equal = h_base1 == h_purge
deterministic = pair_equal and clean_equal and purge_equal

cause = ""
if not deterministic:
    # Heuristic deterministic-cause classification.
    patterns = [
        (r"(SOURCE_DATE_EPOCH|timestamp|time zone|date)", "timestamp_or_timezone"),
        (r"(tar|archive|order)", "tar_or_archive_order"),
        (r"(gcc|clang|rustc|go version|compiler)", "compiler_or_toolchain"),
    ]
    logs = []
    for phase in ("baseline1", "baseline2", "clean_cache", "purge_cache"):
        p = summary.parent / "logs" / f"{tool}.{phase}.log"
        if p.exists():
            logs.append(p.read_text(encoding="utf-8", errors="ignore"))
    body = "\n".join(logs)
    for pat, label in patterns:
        if re.search(pat, body, flags=re.IGNORECASE):
            cause = label
            break
    if not cause:
        cause = "unknown_or_external_dependency_drift"

data = json.loads(summary.read_text(encoding="utf-8"))
data["items"].append(
    {
        "tool": tool,
        "def_path": def_path,
        "hashes": {
            "baseline1": h_base1,
            "baseline2": h_base2,
            "clean_cache": h_clean,
            "purge_cache": h_purge,
        },
        "checks": {
            "same_cache_twice": pair_equal,
            "clean_cache_match": clean_equal,
            "purge_cache_match": purge_equal,
        },
        "deterministic": deterministic,
        "nondeterministic_cause": cause,
    }
)
summary.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
done < <(python3 - "$defs_json" <<'PY'
import json, sys
d = json.loads(open(sys.argv[1], "r", encoding="utf-8").read())
for row in d["defs"]:
    print(f'{row["tool"]}\t{row["path"]}')
PY
)

python3 - "$summary_json" "$POLICY_TOML" "$DOC_REPORT" <<'PY'
import hashlib
import json
import sys
from pathlib import Path
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

summary_path = Path(sys.argv[1])
policy_path = Path(sys.argv[2])
doc_path = Path(sys.argv[3])
data = json.loads(summary_path.read_text(encoding="utf-8"))
policy = tomllib.loads(policy_path.read_text(encoding="utf-8"))
items = data.get("items", [])

total_checks = len(items) * 3
passed_checks = 0
for row in items:
    checks = row.get("checks", {})
    passed_checks += int(bool(checks.get("same_cache_twice")))
    passed_checks += int(bool(checks.get("clean_cache_match")))
    passed_checks += int(bool(checks.get("purge_cache_match")))

confidence = 1.0 if total_checks == 0 else passed_checks / total_checks
threshold = float(policy.get("confidence_min", 1.0))
require_all = bool(policy.get("require_all_tools_deterministic", True))
all_tools_ok = all(bool(r.get("deterministic")) for r in items)
ok = confidence >= threshold and (all_tools_ok if require_all else True)

data["confidence"] = confidence
data["confidence_total_checks"] = total_checks
data["confidence_passed_checks"] = passed_checks
data["ok"] = ok
summary_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = [
    "<!-- Generated by scripts/containers/run-apptainer-frontend-reproducibility.sh -->",
    "",
    "# Apptainer Frontend Reproducibility Report",
    "",
    f"- host: `{data.get('host', '')}`",
    f"- seed: `{data.get('seed', '')}`",
    f"- sampled_tools: `{len(items)}`",
    f"- confidence: `{confidence:.4f}` (threshold `{threshold:.4f}`)",
    f"- all_tools_deterministic_required: `{str(require_all).lower()}`",
    f"- gate_status: `{'PASS' if ok else 'FAIL'}`",
    "",
    "| tool | same_cache_twice | clean_cache_match | purge_cache_match | deterministic | cause_if_mismatch |",
    "|---|---:|---:|---:|---:|---|",
]
for row in sorted(items, key=lambda x: x.get("tool", "")):
    c = row.get("checks", {})
    lines.append(
        f"| `{row.get('tool','')}` | `{str(bool(c.get('same_cache_twice'))).lower()}` | "
        f"`{str(bool(c.get('clean_cache_match'))).lower()}` | `{str(bool(c.get('purge_cache_match'))).lower()}` | "
        f"`{str(bool(row.get('deterministic'))).lower()}` | `{row.get('nondeterministic_cause','')}` |"
    )

doc_path.parent.mkdir(parents=True, exist_ok=True)
doc_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(summary_path)
print(doc_path)
print("frontend reproducibility: " + ("OK" if ok else "FAILED"))
if not ok:
    raise SystemExit(1)
PY

if [[ "$STRICT" != "1" ]]; then
  exit 0
fi

echo "frontend apptainer reproducibility: OK"
