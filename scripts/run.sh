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
    if not name.endswith(".sh") or name == "make.sh":
        continue
    cmd = name[:-3]
    ci = e.get("ci_allowed", "false")
    print(f"{grp}\t{cmd}\tci_allowed={ci}")
PY
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

target="${SCRIPT_DIR}/${group}/make.sh"
if [[ ! -x "$target" ]]; then
  if [[ -f "$target" ]]; then
    chmod +x "$target"
  else
    echo "group dispatcher missing: ${target#"$ROOT_DIR/"}" >&2
    exit 1
  fi
fi

start_epoch="$(date +%s)"
start_iso="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
"$target" "$command" "$@" || rc=$?
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
