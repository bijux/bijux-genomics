#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

DOC="$ROOT_DIR/containers/docs/IMPUTATION_NETWORK_POLICY.md"
if [[ ! -f "$DOC" ]]; then
  echo "missing $DOC" >&2
  exit 1
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
tools = ["glimpse", "impute5", "minimac4", "shapeit5", "beagle", "eagle", "bcftools", "plink2"]
errors = []
for t in tools:
    path = root / "containers/network" / f"{t}.network.toml"
    if not path.exists():
        errors.append(f"missing network metadata: {path.relative_to(root)}")
        continue
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    if bool(data.get("runtime_network", True)):
        errors.append(f"{t}: runtime_network must be false")
if errors:
    print("imputation network policy: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("imputation network policy: OK")
PY
