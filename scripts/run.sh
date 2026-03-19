#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'EOF'
Usage: scripts/run.sh <group> <command> [args...]
       scripts/run.sh --list
EOF
}

list_supported() {
  python3 - "$ROOT_DIR/scripts/SUPPORTED.toml" <<'PY'
import sys
spec = sys.argv[1]
entries = []
cur = None
for raw in open(spec, "r", encoding="utf-8"):
    line = raw.strip()
    if line == "[[script]]":
        if cur:
            entries.append(cur)
        cur = {}
        continue
    if cur is None or "=" not in line:
        continue
    k, v = line.split("=", 1)
    k = k.strip()
    v = v.strip()
    if v.startswith('"') and v.endswith('"'):
        v = v[1:-1]
    elif v in ("true", "false"):
        v = v
    cur[k] = v
if cur:
    entries.append(cur)

for e in sorted(entries, key=lambda x: (x.get("path", ""), x.get("id", ""))):
    path = e.get("path", "")
    if not path.startswith("scripts/"):
        continue
    rel = path[len("scripts/"):]
    if "/" not in rel:
        continue
    grp, name = rel.split("/", 1)
    if grp in ("_lib", "experimental"):
        continue
    if grp in ("containers", "domain", "docs", "examples", "hpc", "lab", "smoke", "test"):
        continue
    if not name.endswith(".sh") or name == "make.sh":
        continue
    cmd = name[:-3]
    ci = e.get("ci_allowed", "false")
    print(f"{grp}\t{cmd}\tci_allowed={ci}")
PY
  for group in docs examples hpc lab smoke test; do
    cargo run -q -p bijux-dev-dna -- "$group" list | while IFS= read -r command_id; do
      [[ -n "$command_id" ]] || continue
      printf '%s\t%s\tci_allowed=true\n' "$group" "$command_id"
    done
  done
  cargo run -q -p bijux-dev-dna -- assets list | while IFS= read -r command_id; do
    [[ -n "$command_id" ]] || continue
    printf 'assets\t%s\tci_allowed=true\n' "$command_id"
  done
  cargo run -q -p bijux-dev-dna -- domain list | while IFS= read -r command_id; do
    [[ -n "$command_id" ]] || continue
    printf 'domain\t%s\tci_allowed=true\n' "$command_id"
  done
  cargo run -q -p bijux-dev-dna -- containers list | while IFS= read -r command_id; do
    [[ -n "$command_id" ]] || continue
    printf 'containers\t%s\tci_allowed=true\n' "$command_id"
  done
  cargo run -q -p bijux-dev-dna -- checks list | while IFS= read -r check_id; do
    [[ -n "$check_id" ]] || continue
    printf 'checks\t%s\tci_allowed=true\n' "$check_id"
  done
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ "${1:-}" == "--list" ]]; then
  list_supported
  exit 0
fi

if [[ $# -lt 2 ]]; then
  usage >&2
  exit 2
fi

group="$1"
command="$2"
shift 2

case "$group" in
  checks|tooling|containers|assets|docs|domain|smoke|test|lab|examples|hpc) ;;
  *)
    echo "unsupported group: $group" >&2
    exit 2
    ;;
esac

start_epoch="$(date +%s)"
start_iso="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if [[ "$group" == "checks" ]]; then
  if [[ "$command" == "--all" || "$command" == "all" ]]; then
    cargo run -p bijux-dev-dna -- checks run --all "$@" || rc=$?
  else
    cargo run -p bijux-dev-dna -- checks run "$command" "$@" || rc=$?
  fi
elif [[ "$group" == "domain" ]]; then
  cargo run -p bijux-dev-dna -- domain run "$command" -- "$@" || rc=$?
elif [[ "$group" == "containers" ]]; then
  cargo run -p bijux-dev-dna -- containers run "$command" -- "$@" || rc=$?
elif [[ "$group" == "assets" ]]; then
  cargo run -p bijux-dev-dna -- assets run "$command" -- "$@" || rc=$?
elif [[ "$group" == "docs" || "$group" == "examples" || "$group" == "hpc" || "$group" == "lab" || "$group" == "smoke" || "$group" == "test" ]]; then
  native_command="$command"
  case "$group/$command" in
    hpc/pull|hpc/lunarc/pull) native_command="lunarc-pull" ;;
    hpc/push|hpc/lunarc/push) native_command="lunarc-push" ;;
    lab/run_bench) native_command="run-bench" ;;
    lab/run_pipelines) native_command="run-pipelines" ;;
    smoke/smoke_bam) native_command="smoke-bam" ;;
    smoke/smoke_fastq) native_command="smoke-fastq" ;;
    test/toy_runs) native_command="toy-runs" ;;
  esac
  cargo run -p bijux-dev-dna -- "$group" run "$native_command" -- "$@" || rc=$?
else
  target="${SCRIPT_DIR}/${group}/make.sh"
  if [[ ! -x "$target" ]]; then
    if [[ -f "$target" ]]; then
      chmod +x "$target"
    else
      echo "group dispatcher missing: ${target#"$ROOT_DIR/"}" >&2
      exit 1
    fi
  fi
  "$target" "$command" "$@" || rc=$?
fi
rc="${rc:-0}"
status="ok"
if [[ "$rc" -ne 0 ]]; then
  status="fail"
fi
end_epoch="$(date +%s)"
end_iso="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
duration_s=$((end_epoch - start_epoch))

timing_dir="${ARTIFACT_DIR:-${ISO_ROOT:-$ROOT_DIR/artifacts}/timing}"
ensure_artifacts_dir "$timing_dir"
timing_file="$timing_dir/${group}__${command//\//_}.json"
python3 - "$timing_file" <<PY
import json
from collections import OrderedDict
payload = OrderedDict([
  ("group", "$group"),
  ("command", "$command"),
  ("status", "$status"),
  ("exit_code", $rc),
  ("start_utc", "$start_iso"),
  ("end_utc", "$end_iso"),
  ("duration_seconds", $duration_s),
])
with open("$timing_file", "w", encoding="utf-8") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\\n")
PY

exit "$rc"
