#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

DOC="$ROOT_DIR/containers/docs/IMPUTATION_RUNTIME_CONSTRAINTS.md"
if [[ ! -f "$DOC" ]]; then
  echo "missing $DOC" >&2
  exit 1
fi

python3 - "$DOC" <<'PY'
from pathlib import Path
import re
import sys

doc = Path(sys.argv[1]).read_text(encoding="utf-8")
tools = ["glimpse", "impute5", "minimac4", "shapeit5", "beagle", "eagle", "bcftools", "plink2"]
errors = []
for t in tools:
    m = re.search(rf"^\|\s*`{re.escape(t)}`\s*\|", doc, flags=re.M)
    if not m:
        errors.append(f"missing constraints row for {t}")
if "cpu_threads_min" not in doc or "ram_gb_min" not in doc or "scratch_gb_min" not in doc:
    errors.append("constraints columns cpu_threads_min/ram_gb_min/scratch_gb_min are required")
if errors:
    print("imputation runtime constraints: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("imputation runtime constraints: OK")
PY
