#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

OUT="${1:-$ROOT_DIR/docs/20-science/TOOL_INDEX.md}"
REG1="$ROOT_DIR/configs/ci/registry/tool_registry.toml"
REG2="$ROOT_DIR/configs/ci/registry/tool_registry_vcf.toml"
REG3="$ROOT_DIR/configs/ci/registry/tool_registry_experimental.toml"
REG4="$ROOT_DIR/configs/ci/registry/tool_registry_vcf_downstream.toml"
SUMMARY_JSON="$ROOT_DIR/artifacts/containers/summary.json"

python3 - <<'PY' "$REG1" "$REG2" "$REG3" "$REG4" "$SUMMARY_JSON" "$OUT"
from pathlib import Path
import json
import sys

reg_paths = [Path(p) for p in sys.argv[1:5]]
summary_path = Path(sys.argv[5])
out = Path(sys.argv[6])

# Minimal TOML-like parser for the generated registries we own.
def parse_tools(path: Path):
    rows = []
    if not path.exists():
        return rows
    cur = None
    for raw in path.read_text(encoding='utf-8').splitlines():
        line = raw.strip()
        if not line or line.startswith('#'):
            continue
        if line == '[[tools]]':
            if cur:
                rows.append(cur)
            cur = {}
            continue
        if cur is None or '=' not in line:
            continue
        k, v = [x.strip() for x in line.split('=', 1)]
        if v.startswith('[') and v.endswith(']'):
            items = [i.strip().strip('"') for i in v[1:-1].split(',') if i.strip()]
            cur[k] = items
        else:
            cur[k] = v.strip('"')
    if cur:
        rows.append(cur)
    return rows

tools = {}
vcf_downstream = {}
for p in reg_paths:
    for t in parse_tools(p):
        tool_id = t.get('id') or t.get('tool_id')
        if not tool_id:
            continue
        tools[tool_id] = {
            'purpose': t.get('tool_role', 'unknown'),
            'stages': t.get('stage_ids', []),
            'container_ref': t.get('container_ref', '-'),
            'citation': t.get('citation', 'TBD') or 'TBD',
            'status': t.get('status', 'unknown'),
            'version': t.get('version', '-'),
        }
        if str(t.get("domain", "")) == "vcf" and any(s.startswith("vcf.") for s in t.get("stage_ids", [])):
            vcf_downstream[tool_id] = {
                "status": t.get("status", "unknown"),
                "stages": t.get("stage_ids", []),
            }

self_reports = {}
if summary_path.exists():
    try:
        summary = json.loads(summary_path.read_text(encoding="utf-8"))
        for item in summary.get("items", []):
            tool = item.get("tool")
            manifest_path = item.get("manifest")
            if not tool or not manifest_path:
                continue
            mp = Path(manifest_path)
            if not mp.exists():
                continue
            try:
                manifest = json.loads(mp.read_text(encoding="utf-8"))
            except Exception:
                continue
            sr_path = manifest.get("self_report_path")
            if not sr_path:
                continue
            sp = Path(sr_path)
            if not sp.exists():
                continue
            try:
                sr = json.loads(sp.read_text(encoding="utf-8"))
            except Exception:
                continue
            self_reports[tool] = sr
    except Exception:
        pass

lines = []
lines.append('<!-- GENERATED FILE - DO NOT EDIT -->')
lines.append('<!-- Regenerate with: scripts/tooling/generate-tool-index.sh -->')
lines.append('')
lines.append('# TOOL_INDEX')
lines.append('')
lines.append('## Purpose')
lines.append('Generated index of registry tools with stage bindings and container references/self-reports.')
lines.append('')
lines.append('## Scope')
lines.append('Source of truth = registry contracts + `artifacts/containers/summary.json` self-reports when available.')
lines.append('')
lines.append('## Non-goals')
lines.append('- Replacing full scientific method docs for each domain.')
lines.append('')
lines.append('## Contracts')
lines.append('- Manual edits are forbidden; regenerate via script.')
lines.append('- Source of truth is registry + containers; this file is a rendered view.')
lines.append('- Tool admission policy is documented in `docs/50-reference/TOOL_ADMISSION.md`.')
lines.append('')
lines.append('See also: [Tool Admission](../50-reference/TOOL_ADMISSION.md)')
lines.append('See also: [VCF Downstream Roadmap](vcf/ROADMAP.md)')
lines.append('')
lines.append('## VCF Downstream / IBD Toolkit')
lines.append('')
for tid in sorted(vcf_downstream):
    info = vcf_downstream[tid]
    stages = ", ".join(info["stages"]) if info["stages"] else "-"
    lines.append(f"- `{tid}` ({info['status']}) : {stages}")
lines.append('')
lines.append('| Tool ID | Purpose | Stage Bindings | Container Ref | Version | Citation | Status |')
lines.append('|---|---|---|---|---|---|---|')
for tool_id in sorted(tools):
    row = tools[tool_id]
    stages = ', '.join(row['stages']) if row['stages'] else '-'
    version = row['version']
    if tool_id in self_reports:
        version = str(self_reports[tool_id].get("version", version))
    lines.append(
        f"| `{tool_id}` | `{row['purpose']}` | `{stages}` | `{row['container_ref']}` | `{version}` | {row['citation']} | `{row['status']}` |"
    )

out.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'generated {out}')
PY
