#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

doc="${ROOT_DIR}/docs/10-architecture/SSOT.md"
if ! rg -q 'domain/\*/\*\*/\*\.yaml.*source of truth' "$doc"; then
  echo "ssot authority check: docs/10-architecture/SSOT.md must declare domain/*/**/*.yaml as source of truth" >&2
  exit 1
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
errors = []
for domain_dir in sorted((root / "domain").iterdir()):
    if not domain_dir.is_dir():
        continue
    index_path = domain_dir / "index.yaml"
    if not index_path.exists():
        continue
    text = index_path.read_text(encoding="utf-8")
    m = re.search(r'^domain_version:\s*"?([^"\n#]+)"?\s*$', text, flags=re.MULTILINE)
    if not m:
        errors.append(f"{index_path.relative_to(root)} missing domain_version: v1|v2")
        continue
    version = m.group(1).strip()
    if version not in {"v1", "v2"}:
        errors.append(f"{index_path.relative_to(root)} has invalid domain_version '{version}' (expected v1|v2)")
    if domain_dir.name == "vcf" and version != "v2":
        errors.append("domain/vcf/index.yaml must declare domain_version: v2")

if errors:
    print("ssot authority check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("ssot authority/version: OK")
PY
