#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT
"$ROOT_DIR/scripts/checks/generate-vcf-compatibility-matrix.sh" >/dev/null
cp "$ROOT_DIR/docs/50-reference/VCF_DOWNSTREAM_COMPATIBILITY_MATRIX.md" "$TMP"
git -C "$ROOT_DIR" diff --quiet -- docs/50-reference/VCF_DOWNSTREAM_COMPATIBILITY_MATRIX.md || {
  echo "vcf compatibility matrix: stale; regenerate with scripts/checks/generate-vcf-compatibility-matrix.sh" >&2
  exit 1
}
python3 - "$TMP" <<'PY'
import sys
from pathlib import Path
text = Path(sys.argv[1]).read_text(encoding="utf-8")
rows = [ln for ln in text.splitlines() if ln.startswith("| ")]
if len(rows) <= 2:
    raise SystemExit("vcf compatibility matrix: missing data rows")
print("vcf compatibility matrix: OK")
PY
