#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
import pathlib
import sys

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = pathlib.Path(sys.argv[1])
reg_paths = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
images_toml = root / "configs/ci/tools/images.toml"

def load_toml(path):
    return tomllib.loads(path.read_text(encoding="utf-8"))

errors = []
tools = []
for reg in reg_paths:
    data = load_toml(reg)
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        status = str(row.get("status") or "").strip()
        if status == "planned":
            continue
        if status != "production":
            continue
        if not bool(row.get("container", False)):
            continue
        tools.append((reg, row))

images = load_toml(images_toml)
parity_exemptions = set()
for key in ("parity_exemptions", "apptainer_parity_exemptions"):
    table = images.get(key, {})
    if isinstance(table, dict):
        for tool_id, enabled in table.items():
            if bool(enabled):
                parity_exemptions.add(str(tool_id))

for reg, tool in tools:
    tool_id = str(tool.get("id") or tool.get("tool_id") or "<unknown>")
    runtimes = {str(x) for x in tool.get("runtimes", [])}
    dockerfile = str(tool.get("dockerfile", "")).strip()
    apptainer_def = str(tool.get("apptainer_def", "")).strip()

    docker_exists = False
    if dockerfile:
        docker_path = root / dockerfile
        docker_exists = docker_path.exists()
        if not docker_exists:
            errors.append(f"{reg}: {tool_id} dockerfile missing: {dockerfile}")
        expected = f"Dockerfile.{tool_id}"
        if pathlib.Path(dockerfile).name != expected:
            errors.append(f"{reg}: {tool_id} dockerfile naming mismatch: expected {expected}")

    appt_exists = False
    if apptainer_def:
        appt_path = root / apptainer_def
        appt_exists = appt_path.exists()
        if not appt_exists:
            errors.append(f"{reg}: {tool_id} apptainer def missing: {apptainer_def}")
        expected = f"{tool_id}.def"
        if pathlib.Path(apptainer_def).name != expected:
            errors.append(f"{reg}: {tool_id} apptainer naming mismatch: expected {expected}")

    if "docker" in runtimes and not dockerfile:
        errors.append(f"{reg}: {tool_id} runtime includes docker but dockerfile is unset")
    if "apptainer" in runtimes and not apptainer_def:
        errors.append(f"{reg}: {tool_id} runtime includes apptainer but apptainer_def is unset")
    if not (dockerfile or apptainer_def):
        errors.append(f"{reg}: {tool_id} supported container tool has no container paths")

    if dockerfile and not apptainer_def and tool_id not in parity_exemptions:
        errors.append(
            f"{reg}: {tool_id} has dockerfile but no apptainer_def and is not exempt "
            f"(set configs/ci/tools/images.toml [parity_exemptions].{tool_id} = true)"
        )

if errors:
    print("tool/container coverage check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    sys.exit(1)

print("tool/container coverage: OK")
PY
