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
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
bundles = tomllib.loads((root / "configs/ci/tools/toolkit_bundles.toml").read_text(encoding="utf-8")).get("bundles", {})
vcf_bundle = bundles.get("vcf_downstream", {})
tools = set(vcf_bundle.get("tools", []))

domain_stages = {p.stem for p in (root / "domain/vcf/stages").glob("*.yaml")}
vcf_downstream_enabled = any(s in domain_stages for s in ("phasing", "imputation"))
if not vcf_downstream_enabled:
    print("vcf downstream bundle coverage: SKIP (no downstream phasing/imputation stages)")
    raise SystemExit(0)

phasing_required = {"beagle", "eagle", "shapeit5"}
imputation_required = {"beagle", "impute5", "minimac4", "glimpse"}

errors = []
if not (tools & phasing_required):
    errors.append(f"vcf_downstream bundle requires at least one phasing tool from {sorted(phasing_required)}")
if not (tools & imputation_required):
    errors.append(f"vcf_downstream bundle requires at least one imputation tool from {sorted(imputation_required)}")

if errors:
    print("vcf downstream bundle coverage: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("vcf downstream bundle coverage: OK")
PY
