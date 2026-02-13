#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/docs/20-science/TOOL_INDEX.md"
REG1="$ROOT/configs/ci/registry/tool_registry.toml"
REG2="$ROOT/configs/ci/registry/tool_registry_vcf.toml"
REG3="$ROOT/configs/ci/registry/tool_registry_experimental.toml"

python3 - <<'PY' "$REG1" "$REG2" "$REG3" "$OUT"
from pathlib import Path
import sys

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib  # type: ignore

reg_paths = [Path(p) for p in sys.argv[1:4]]
out = Path(sys.argv[4])

tools = {}
for path in reg_paths:
    if not path.exists():
        continue
    data = tomllib.loads(path.read_text(encoding='utf-8'))
    for row in data.get('tools', []):
        tool_id = row.get('id') or row.get('tool_id')
        if not tool_id:
            continue
        stages = row.get('stage_ids') or []
        entry = tools.setdefault(tool_id, set())
        for s in stages:
            if isinstance(s, str) and s:
                entry.add(s)

lines = []
lines.append('<!-- GENERATED FILE - DO NOT EDIT -->')
lines.append('<!-- Regenerate with: scripts/tooling/generate-tool-index.sh -->')
lines.append('')
lines.append('# TOOL_INDEX')
lines.append('')
lines.append('Generated from `configs/ci/registry/tool_registry*.toml`.')
lines.append('')
lines.append('## Tools')
for tool in sorted(tools):
    stages = ', '.join(sorted(tools[tool])) if tools[tool] else '(no stages)'
    lines.append(f'- `{tool}`: {stages}')
lines.append('')
out.write_text('\n'.join(lines), encoding='utf-8')
print(f'generated {out}')
PY
