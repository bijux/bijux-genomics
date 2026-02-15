#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo "Usage: scripts/checks/check-reference-service-boundary.sh"
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
scan_roots = [root / "crates/bijux-dna-stages-vcf/src/pipeline_sections/execution"]

ref_usage = re.compile(r"reference_fasta|reference_path|reference\s*=", re.IGNORECASE)
direct_path = re.compile(r"Path::new\(|PathBuf::from\(|std::fs::read_to_string\(")
service_markers = (
    "ref_service(",
    "resolve_reference_bundle(",
    "resolve_reference_bank(",
    "resolve_species_authority(",
)

violations = []
for base in scan_roots:
    for file in base.rglob("*.rs"):
        text = file.read_text(encoding="utf-8")
        if "executor" not in str(file) and "call_filter_and_gl" not in str(file):
            continue
        if not ref_usage.search(text):
            continue
        if not direct_path.search(text):
            continue
        if any(marker in text for marker in service_markers):
            continue
        rel = file.relative_to(root)
        violations.append(str(rel))

if violations:
    print("reference-service-boundary: FAILED", file=sys.stderr)
    print("Executors touching reference paths must resolve via bijux-dna-db-ref service API.", file=sys.stderr)
    for item in violations:
        print(f"- {item}", file=sys.stderr)
    raise SystemExit(1)

print("reference-service-boundary: OK")
PY
