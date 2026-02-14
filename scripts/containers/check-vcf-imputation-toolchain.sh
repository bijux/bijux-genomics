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
core = ["glimpse", "impute5", "minimac4", "shapeit5", "beagle", "eagle", "bcftools", "plink2"]
required_path = root / "configs/ci/tools/required_tools_vcf_downstream.toml"
registry_path = root / "configs/ci/registry/tool_registry_vcf_downstream.toml"
registry_vcf_path = root / "configs/ci/registry/tool_registry_vcf.toml"
required = tomllib.loads(required_path.read_text(encoding="utf-8")).get("required_tools", [])
registry = tomllib.loads(registry_path.read_text(encoding="utf-8")).get("tools", [])
registry_vcf = tomllib.loads(registry_vcf_path.read_text(encoding="utf-8")).get("tools", [])
errors = []

required_set = set(required)
registry_ids = {str(r.get("id", "")).strip() for r in registry if isinstance(r, dict)}
if required_set != registry_ids:
    missing_in_required = sorted(registry_ids - required_set)
    missing_in_registry = sorted(required_set - registry_ids)
    if missing_in_required:
        errors.append(f"required_tools_vcf_downstream missing registry ids: {missing_in_required}")
    if missing_in_registry:
        errors.append(f"required_tools_vcf_downstream has unknown ids: {missing_in_registry}")

rows = {str(r.get("id", "")).strip(): r for r in registry if isinstance(r, dict)}
rows_vcf = {str(r.get("id", "")).strip(): r for r in registry_vcf if isinstance(r, dict)}
for tool in core:
    row = rows.get(tool) or rows_vcf.get(tool)
    if not row:
        errors.append(f"{tool}: missing in VCF registry surfaces")
        continue
    if not bool(row.get("container", False)):
        errors.append(f"{tool}: container=false in vcf downstream registry")
    runtimes = set(row.get("runtimes", []))
    if "docker" not in runtimes or "apptainer" not in runtimes:
        errors.append(f"{tool}: runtimes must include docker+apptainer, got {sorted(runtimes)}")
    for key in ("smoke_version_cmd", "smoke_help_cmd", "version_cmd", "help_cmd", "expected_bin"):
        if not str(row.get(key, "")).strip():
            errors.append(f"{tool}: missing {key}")
    dockerfile = str(row.get("dockerfile", "")).strip()
    apptainer_def = str(row.get("apptainer_def", "")).strip()
    if not dockerfile or not (root / dockerfile).exists():
        errors.append(f"{tool}: dockerfile missing: {dockerfile or '<empty>'}")
    if not apptainer_def or not (root / apptainer_def).exists():
        errors.append(f"{tool}: apptainer_def missing: {apptainer_def or '<empty>'}")
    license_file = root / "containers/licenses" / f"{tool}.license.toml"
    if not license_file.exists():
        errors.append(f"{tool}: missing license metadata {license_file.relative_to(root)}")
    tool_doc = root / "containers/docs/tools" / f"{tool}.md"
    if not tool_doc.exists():
        errors.append(f"{tool}: missing tool doc {tool_doc.relative_to(root)}")

if errors:
    print("vcf imputation toolchain check: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print(f"vcf imputation toolchain check: OK ({len(core)} core tools)")
PY
