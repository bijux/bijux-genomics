#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
allow = {
    root / "scripts/tooling/acquire-panels.sh",
    root / "scripts/tooling/acquire-maps.sh",
}
network_pat = re.compile(r"(urllib\.request\.urlopen|curl\s|wget\s)")
reference_scope_pat = re.compile(
    r"(configs/vcf/panels|configs/vcf/maps|panels\.toml|maps\.toml|/panels/|/maps/)"
)
errors = []
for rel in ("scripts", "crates"):
    for p in (root / rel).rglob("*"):
        if not p.is_file():
            continue
        if p.suffix in {".png", ".jpg", ".lock", ".snap"}:
            continue
        if p in allow:
            continue
        try:
            text = p.read_text(encoding="utf-8")
        except Exception:
            continue
        if network_pat.search(text) and reference_scope_pat.search(text):
            errors.append(str(p.relative_to(root)))

if errors:
    print("reference-fetch-paths: FAILED", file=sys.stderr)
    print("Only acquire-panels/acquire-maps may include network fetch logic.", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("reference-fetch-paths: OK")
PY
