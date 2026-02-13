#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
external = set(tomllib.loads((root / "configs/domain/external_tools.toml").read_text(encoding="utf-8")).get("non_container_tools", {}).keys())

registry_tools = set()
for reg in (
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
):
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        tool_id = str(row.get("id") or row.get("tool_id") or "").strip()
        if tool_id:
            registry_tools.add(tool_id)

errors = []
for fx in sorted((root / "domain").glob("*/fixtures/*/*.txt")):
    tool = fx.stem
    if tool not in registry_tools and tool not in external:
        errors.append(f"{fx.relative_to(root)}: tool '{tool}' missing from registries and external_tools allowlist")

required = {"gatk","picard","preseq","bamutil","ngsbriggs","dustmasker","seqfu","seqprep","seqpurge","diamond","fastq_scan"}
missing_required = sorted(required - external)
if missing_required:
    errors.append(f"configs/domain/external_tools.toml missing required external markers: {missing_required}")

if errors:
    print("external tool policy check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("external tool policy: OK")
PY
