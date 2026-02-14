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
import re
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
license_dir = root / "containers/licenses"
registry_paths = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
registry_tools = {}
for rp in registry_paths:
    if not rp.exists():
        continue
    data = tomllib.loads(rp.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        if not bool(row.get("container", False)):
            continue
        tool = str(row.get("id") or row.get("tool_id") or "").strip()
        if not tool:
            continue
        registry_tools[tool] = row

versions_path = root / "containers/versions/versions.toml"
versions = tomllib.loads(versions_path.read_text(encoding="utf-8")) if versions_path.exists() else {}
errors = []
for tool, row in sorted(registry_tools.items()):
    meta = license_dir / f"{tool}.license.toml"
    if not meta.exists():
        errors.append(f"missing {meta.relative_to(root)}")
        continue
    data = tomllib.loads(meta.read_text(encoding="utf-8"))
    for key in (
        "tool_id",
        "container_kind",
        "spdx",
        "upstream_license_id",
        "upstream_url",
        "upstream_version",
        "upstream_checksum",
        "redistribution_note",
        "citation",
        "version",
    ):
        if not str(data.get(key, "")).strip():
            errors.append(f"{meta.relative_to(root)} missing key: {key}")
    if data.get("tool_id") != tool:
        errors.append(f"{meta.relative_to(root)} tool_id mismatch")
    up = str(data.get("upstream_url", ""))
    if not up.startswith(("http://", "https://")):
        errors.append(f"{meta.relative_to(root)} upstream_url must be URL")
    checksum = str(data.get("upstream_checksum", "")).strip()
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", checksum):
        errors.append(f"{meta.relative_to(root)} upstream_checksum must be exact sha256:<64hex>")
    if str(data.get("redistribution_note", "")).strip().lower() in {"", "unknown", "n/a"}:
        errors.append(f"{meta.relative_to(root)} redistribution_note must be explicit")

    # Contract for non-bijux provenance.
    apptainer_def = str(row.get("apptainer_def", "")).strip()
    if "/non-bijux/" in apptainer_def:
        vrow = versions.get(tool, {}) if isinstance(versions, dict) else {}
        src = str(vrow.get("source", "")).strip()
        ver = str(vrow.get("version", "")).strip()
        if not src or src != up:
            errors.append(f"{meta.relative_to(root)} non-bijux upstream_url must match versions.toml source")
        if not ver or str(data.get("upstream_version", "")).strip() != ver:
            errors.append(f"{meta.relative_to(root)} non-bijux upstream_version must match versions.toml version")
        if checksum == "sha256:" + ("0" * 64):
            errors.append(f"{meta.relative_to(root)} non-bijux upstream_checksum must not be placeholder zeros")

if errors:
    print("license metadata check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("license metadata: OK")
PY
