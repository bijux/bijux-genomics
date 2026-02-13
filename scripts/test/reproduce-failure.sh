#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

LOG_PATH="${1:-}"
if [[ -z "$LOG_PATH" ]]; then
  echo "usage: $0 <nextest-jsonl-log>" >&2
  exit 2
fi
if [[ ! -f "$LOG_PATH" ]]; then
  echo "missing log file: $LOG_PATH" >&2
  exit 1
fi

python3 - "$LOG_PATH" <<'PY'
import json
import sys
from collections import OrderedDict

path = sys.argv[1]
failures = OrderedDict()
with open(path, 'r', encoding='utf-8') as f:
    for raw in f:
        raw = raw.strip()
        if not raw:
            continue
        try:
            obj = json.loads(raw)
        except json.JSONDecodeError:
            continue
        status = obj.get("status")
        if status not in {"fail", "failed"}:
            continue
        test_name = obj.get("name") or obj.get("test_name") or obj.get("test")
        binary = obj.get("binary") or obj.get("binary_id")
        if not test_name:
            continue
        key = (binary or "", test_name)
        failures[key] = True

for (binary, test_name) in failures.keys():
    if binary:
        print(f"./bin/isolate sh -ceu 'export CARGO_TARGET_DIR=\"$ISO_ROOT/target-test\"; cargo nextest run --test-threads 1 {binary} {test_name}'")
    else:
        print(f"./bin/isolate sh -ceu 'export CARGO_TARGET_DIR=\"$ISO_ROOT/target-test\"; cargo nextest run --test-threads 1 {test_name}'")
PY
