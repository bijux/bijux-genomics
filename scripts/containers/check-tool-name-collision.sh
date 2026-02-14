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
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
regs = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
images = tomllib.loads((root / "configs/ci/tools/images.toml").read_text(encoding="utf-8"))
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
tool_ids = set()
for raw in (root / "containers/TOOL_IDS.txt").read_text(encoding="utf-8").splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    tool_ids.add(line.split("\t", 1)[0])
docker_ids = {p.name.split("Dockerfile.", 1)[1] for p in (root / "containers/docker/arm64").glob("Dockerfile.*")}
apptainer_ids = {p.stem for p in (root / "containers/apptainer/bijux").glob("*.def")}
apptainer_ids |= {p.stem for p in (root / "containers/apptainer/non-bijux").glob("*.def")}
domain_ids = {p.stem for p in (root / "domain").glob("*/tools/*.yaml") if p.stem != "_schema"}

tools = {}
bin_to_tool = {}
for reg in regs:
    if not reg.exists():
        continue
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        tid = str(row.get("id") or row.get("tool_id") or "").strip()
        if not tid:
            continue
        tools[tid] = {
            "expected_bin": str(row.get("expected_bin") or "").strip(),
            "status": str(row.get("status") or "").strip(),
        }
        b = str(row.get("expected_bin") or "").strip()
        if b:
            prev = bin_to_tool.get(b)
            if prev and prev != tid:
                errors.append(f"expected_bin collision: '{b}' used by both '{prev}' and '{tid}'")
            bin_to_tool[b] = tid

errors = []

# Collision rule: if names differ only by a numeric suffix (e.g. plink/plink2),
# both IDs must be present as separate registry/image/version entries.
for tid in sorted(tools):
    m = re.match(r"^([a-z_]+?)(\d+)$", tid)
    if not m:
        continue
    base = m.group(1)
    if base not in tools:
        continue
    for candidate in (base, tid):
        if candidate not in images:
            errors.append(f"name-collision: missing images entry for '{candidate}'")
        if candidate not in versions:
            errors.append(f"name-collision: missing versions entry for '{candidate}'")

    base_bin = tools[base]["expected_bin"]
    suffixed_bin = tools[tid]["expected_bin"]
    if base_bin and suffixed_bin and base_bin == suffixed_bin:
        errors.append(
            f"name-collision: expected_bin must differ for '{base}' and '{tid}' (both '{base_bin}')"
        )

if errors:
    print("tool-name-collision: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

# Cross-surface ID parity and normalization.
surfaces = {
    "registry": set(tools.keys()),
    "images": {k for k, v in images.items() if isinstance(v, dict)},
    "versions": set(versions.keys()),
    "tool_ids": tool_ids,
    "docker": docker_ids,
    "apptainer": apptainer_ids,
    "domain_tools": domain_ids,
}
all_ids = set().union(*surfaces.values())
norm = re.compile(r"^[a-z][a-z0-9_]*$")
for sid in sorted(all_ids):
    if not norm.fullmatch(sid):
        errors.append(f"id normalization: '{sid}' is not snake_case")

for sid in sorted(all_ids):
    present = [name for name, vals in surfaces.items() if sid in vals]
    # domain tools may be external; but every non-domain surface id must exist in registry.
    if "registry" not in present and any(s in present for s in ("images", "versions", "tool_ids", "docker", "apptainer")):
        errors.append(f"id parity: '{sid}' present in {present} but missing from registry")

# Documentation parity: name map must include all registry tool ids.
name_map = root / "containers/docs/TOOL_NAME_MAP.md"
if not name_map.exists():
    errors.append("missing containers/docs/TOOL_NAME_MAP.md")
else:
    text = name_map.read_text(encoding="utf-8")
    for tid in sorted(tools):
        if f"`{tid}`" not in text:
            errors.append(f"tool-name-map missing tool id '{tid}'")

if errors:
    print("tool-name-collision: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("tool-name-collision: OK")
PY
