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
  echo "Usage: scripts/checks/check-reference-path-governance.sh"
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
forbidden_abs = re.compile(
    r"\"/(Users|home|tmp|var|opt|mnt|data)/[^\"\\n]*(reference|ref|fasta|genome)",
    re.IGNORECASE,
)

violations = []
crate_scopes = [
    root / "crates/bijux-dna-stages-vcf/src",
    root / "crates/bijux-dna-stages-bam/src",
    root / "crates/bijux-dna-stages-fastq/src",
    root / "crates/bijux-dna-api/src/internal/handlers",
]

for scope in crate_scopes:
    for file in scope.rglob("*.rs"):
        text = file.read_text(encoding="utf-8")
        if forbidden_abs.search(text):
            violations.append(str(file.relative_to(root)))

for file in (root / "scripts").rglob("*.sh"):
    rel = str(file.relative_to(root))
    text = file.read_text(encoding="utf-8")
    if forbidden_abs.search(text):
        violations.append(rel)

if violations:
    print("reference-path-governance: FAILED", file=sys.stderr)
    print("Uncontrolled reference filesystem literals detected; use db-ref authority + acquire-reference layout.", file=sys.stderr)
    for item in sorted(set(violations)):
        print(f"- {item}", file=sys.stderr)
    raise SystemExit(1)

print("reference-path-governance: OK")
PY
