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

python3 - <<'PY' "$REG1" "$REG2" "$REG3" "$OUT"
from pathlib import Path
import re
import sys

reg_paths = [Path(p) for p in sys.argv[1:4]]
out = Path(sys.argv[4])

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
        }

lines = []
lines.append('<!-- GENERATED FILE - DO NOT EDIT -->')
lines.append('<!-- Regenerate with: scripts/tooling/generate-tool-index.sh -->')
lines.append('')
lines.append('# TOOL_INDEX')
lines.append('')
lines.append('## Purpose')
lines.append('Generated index of registry tools with stage bindings and container references.')
lines.append('')
lines.append('## Scope')
lines.append('Derived from `configs/ci/registry/tool_registry*.toml`.')
lines.append('')
lines.append('## Non-goals')
lines.append('- Replacing full scientific method docs for each domain.')
lines.append('')
lines.append('## Contracts')
lines.append('- Manual edits are forbidden; regenerate via script.')
lines.append('')
lines.append('| Tool ID | Purpose | Stage Bindings | Container Ref | Citation | Status |')
lines.append('|---|---|---|---|---|---|')
for tool_id in sorted(tools):
    row = tools[tool_id]
    stages = ', '.join(row['stages']) if row['stages'] else '-'
    lines.append(
        f"| `{tool_id}` | `{row['purpose']}` | `{stages}` | `{row['container_ref']}` | {row['citation']} | `{row['status']}` |"
    )

out.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'generated {out}')
PY
