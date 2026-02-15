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
  cat <<'EOF'
Usage: scripts/tooling/crash-triage.sh <crash_provenance.json>
Prints top likely crash causes from crash provenance.
EOF
}

[[ $# -ge 1 ]] || { usage >&2; exit 2; }
[[ "$1" == "--help" || "$1" == "-h" ]] && { usage; exit 0; }

path="$1"
[[ -f "$path" ]] || { echo "crash-triage: missing file $path" >&2; exit 1; }

python3 - "$path" <<'PY'
import json
import re
import sys
from pathlib import Path

p = Path(sys.argv[1])
data = json.loads(p.read_text(encoding="utf-8"))
stderr_lines = data.get("stderr_last_lines", []) or []
stderr = "\n".join(stderr_lines).lower()
command = str(data.get("command", "")).lower()
exit_code = data.get("exit_code")

causes = []
def add(score, code, msg):
    causes.append((score, code, msg))

if "no such file" in stderr or "cannot open" in stderr:
    add(100, "input_missing", "Input file missing/unreadable.")
if "index" in stderr and ("missing" in stderr or "failed" in stderr):
    add(95, "index_missing", "Index missing or invalid.")
if "out of memory" in stderr or "cannot allocate memory" in stderr or "killed" in stderr:
    add(90, "resource_exhausted", "Process likely hit memory limit.")
if "header" in stderr or "contig" in stderr or "chromosome" in stderr:
    add(85, "reference_mismatch", "Header/contig/reference mismatch.")
if "not compressed" in stderr and ("tabix" in command or "bgzip" in command):
    add(80, "compression_contract", "Expected bgzip-compressed input for indexing.")
if exit_code in (126, 127):
    add(75, "runner_contract", "Command/image contract issue (missing binary or exec failure).")
if not causes:
    add(10, "unknown", "No high-confidence pattern found; inspect full logs.")

causes.sort(reverse=True)
print("crash-triage: top causes")
for _, code, msg in causes[:5]:
    print(f"- {code}: {msg}")
PY
