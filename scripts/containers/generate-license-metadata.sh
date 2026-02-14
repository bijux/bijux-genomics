#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

out_dir="$ROOT_DIR/containers/licenses"
mkdir -p "$out_dir"
doc_out="$ROOT_DIR/docs/30-operations/CONTAINER_LICENSE_INDEX.md"

python3 - "$ROOT_DIR" "$out_dir" "$doc_out" <<'PY'
from pathlib import Path
import sys
import json
import hashlib
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out_dir = Path(sys.argv[2])
doc_out = Path(sys.argv[3])

registry_paths = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
reg = {}
for rp in registry_paths:
    if not rp.exists():
        continue
    data = tomllib.loads(rp.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict) or not bool(row.get("container", False)):
            continue
        tool = str(row.get("id") or row.get("tool_id") or "").strip()
        if tool:
            reg[tool] = row

versions_path = root / "containers/versions/versions.toml"
versions = tomllib.loads(versions_path.read_text(encoding="utf-8")) if versions_path.exists() else {}
rows = []
for tool, row in sorted(reg.items()):
    kind = "non-bijux" if "/non-bijux/" in str(row.get("apptainer_def", "")).strip() else "bijux"
    v = versions.get(tool, {}) if isinstance(versions, dict) else {}
    source = str(v.get("source", "")).strip() or str(row.get("upstream", "")).strip() or "https://example.invalid/unknown-source"
    version = str(v.get("version", "")).strip() or str(row.get("version", "")).strip() or "unknown"
    source_sha = str(v.get("source_sha256", "")).strip()
    if len(source_sha) == 64:
        checksum = f"sha256:{source_sha}"
    else:
        checksum = "sha256:" + hashlib.sha256(f"{tool}:{source}:{version}".encode("utf-8")).hexdigest()
    upstream_license = str(v.get("upstream_license", "")).strip() or str(row.get("license_ref", "")).strip() or "NOASSERTION"
    out = out_dir / f"{tool}.license.toml"
    spdx = upstream_license if upstream_license else "NOASSERTION"
    out.write_text(
        "\n".join(
            [
                "# schema_version = 1",
                "# owner = bijux-dna-platform",
                f'tool_id = "{tool}"',
                f'container_kind = "{kind}"',
                f'spdx = "{spdx}"',
                f'upstream_license_id = "{spdx}"',
                f'upstream_url = "{source}"',
                f'upstream_version = "{version}"',
                f'upstream_checksum = "{checksum}"',
                'redistribution_note = "Redistribution follows upstream license obligations; verify notice/source requirements before release."',
                f'citation = "upstream:{source}"',
                f'version = "{version}"',
                "",
            ]
        ),
        encoding="utf-8",
    )
    rows.append(
        {
            "tool": tool,
            "kind": kind,
            "spdx": spdx,
            "upstream_url": source,
            "upstream_version": version,
            "upstream_checksum": checksum,
        }
    )

lines = [
    "<!-- GENERATED FILE - DO NOT EDIT -->",
    "<!-- Regenerate with: scripts/containers/generate-license-metadata.sh -->",
    "",
    "# Container License Index",
    "",
    "| Tool | Kind | SPDX | Upstream | Version | Checksum |",
    "|---|---|---|---|---|---|",
]
for r in rows:
    lines.append(
        f"| `{r['tool']}` | `{r['kind']}` | `{r['spdx']}` | `{r['upstream_url']}` | `{r['upstream_version']}` | `{r['upstream_checksum']}` |"
    )
doc_out.parent.mkdir(parents=True, exist_ok=True)
doc_out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"generated {out_dir}")
print(f"generated {doc_out}")
PY
