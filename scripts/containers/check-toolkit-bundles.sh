#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
Usage: scripts/containers/check-toolkit-bundles.sh
USAGE
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
bundles_path = root / "configs/ci/tools/toolkit_bundles.toml"
images_paths = [root / "configs/ci/tools/images.toml", root / "configs/ci/images.toml"]
registry_paths = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]

if not bundles_path.exists():
    print(f"toolkit bundles: missing {bundles_path}", file=sys.stderr)
    raise SystemExit(1)

bundles = tomllib.loads(bundles_path.read_text(encoding="utf-8")).get("bundles", {})
if not bundles:
    print("toolkit bundles: no [bundles.*] entries found", file=sys.stderr)
    raise SystemExit(1)

registry = {}
for rp in registry_paths:
    if not rp.exists():
        continue
    data = tomllib.loads(rp.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        tool = str(row.get("id", "")).strip()
        if not tool:
            continue
        registry[tool] = row

images = {}
for ip in images_paths:
    if not ip.exists():
        continue
    data = tomllib.loads(ip.read_text(encoding="utf-8"))
    for key, val in data.items():
        if isinstance(val, dict):
            images[key] = val

apptainer_tools = {p.stem for p in (root / "containers/apptainer/bijux").glob("*.def")}
apptainer_tools |= {p.stem for p in (root / "containers/apptainer/non-bijux").glob("*.def")}
docker_tools = {p.name.split("Dockerfile.", 1)[1] for p in (root / "containers/docker/arm64").glob("Dockerfile.*")}

errors = []
for bundle_id, spec in sorted(bundles.items()):
    tools = spec.get("tools", [])
    if not isinstance(tools, list) or not tools:
        errors.append(f"{bundle_id}: tools must be a non-empty array")
        continue
    for tool in tools:
        if tool not in registry:
            errors.append(f"{bundle_id}: tool '{tool}' missing from registry")
            continue
        if tool not in images:
            errors.append(f"{bundle_id}: tool '{tool}' missing images.toml metadata")
            continue
        image_meta = images.get(tool, {})
        if not str(image_meta.get("version", "")).strip():
            errors.append(f"{bundle_id}: tool '{tool}' images.toml entry missing version")
        status = str(registry[tool].get("status", "")).strip()
        if status not in {"production", "experimental", "planned"}:
            errors.append(f"{bundle_id}: tool '{tool}' has unsupported status '{status}'")
            continue
        if status == "planned":
            enabled = image_meta.get("enabled")
            if enabled not in (False, "false"):
                errors.append(f"{bundle_id}: planned tool '{tool}' must be enabled=false in images.toml")
            continue
        policy = str(image_meta.get("shipping_policy", "")).strip()
        has_apptainer = tool in apptainer_tools
        has_docker = tool in docker_tools
        if not policy:
            if has_apptainer and has_docker:
                policy = "docker_apptainer"
            elif has_apptainer:
                policy = "apptainer_only"
            elif has_docker:
                policy = "docker_only"
            else:
                policy = "none"
        if policy == "apptainer_only" and not has_apptainer:
            errors.append(f"{bundle_id}: production tool '{tool}' requires apptainer container")
        elif policy == "docker_only" and not has_docker:
            errors.append(f"{bundle_id}: production tool '{tool}' requires docker container")
        elif policy == "docker_apptainer":
            if not has_apptainer or not has_docker:
                errors.append(f"{bundle_id}: production tool '{tool}' requires both docker and apptainer containers")
        else:
            if not has_apptainer and not has_docker:
                errors.append(f"{bundle_id}: production tool '{tool}' has no container definition")

if errors:
    print("toolkit bundle completeness check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("toolkit bundle completeness: OK")
PY
