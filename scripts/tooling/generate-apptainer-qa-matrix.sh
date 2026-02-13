#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/docs/30-operations/APPTAINER_QA_MATRIX.md}"
REG1="$ROOT_DIR/configs/ci/registry/tool_registry.toml"
REG2="$ROOT_DIR/configs/ci/registry/tool_registry_vcf.toml"
REG3="$ROOT_DIR/configs/ci/registry/tool_registry_experimental.toml"
REG4="$ROOT_DIR/configs/ci/registry/tool_registry_vcf_downstream.toml"
SUMMARY_JSON="$ROOT_DIR/artifacts/containers/summary/summary.json"

python3 - <<'PY' "$REG1" "$REG2" "$REG3" "$REG4" "$SUMMARY_JSON" "$OUT"
from pathlib import Path
import sys
import json

def parse_tools(path: Path):
    rows = []
    if not path.exists():
        return rows
    cur = None
    for raw in path.read_text(encoding='utf-8').splitlines():
        s = raw.strip()
        if not s or s.startswith('#'):
            continue
        if s == '[[tools]]':
            if cur:
                rows.append(cur)
            cur = {}
            continue
        if cur is None or '=' not in s:
            continue
        k, v = [x.strip() for x in s.split('=', 1)]
        if v.startswith('[') and v.endswith(']'):
            items = [i.strip().strip('"') for i in v[1:-1].split(',') if i.strip()]
            cur[k] = items
        else:
            cur[k] = v.strip('"')
    if cur:
        rows.append(cur)
    return rows

regs = [Path(p) for p in sys.argv[1:5]]
summary_json = Path(sys.argv[5])
out = Path(sys.argv[6])
rows = {}
status_from_summary = {}

if summary_json.exists():
    try:
        payload = json.loads(summary_json.read_text(encoding="utf-8"))
        for item in payload.get("items", []):
            tool = item.get("tool")
            runtime = item.get("runtime")
            status = item.get("status")
            if tool and runtime == "apptainer" and status:
                status_from_summary[tool] = status
    except Exception:
        pass
for rp in regs:
    for t in parse_tools(rp):
        tool = t.get('id') or t.get('tool_id')
        if not tool:
            continue
        runtimes = t.get('runtimes', [])
        if 'apptainer' not in runtimes:
            continue
        rows[tool] = {
            'apptainer_def': t.get('apptainer_def', '-'),
            'smoke_version_cmd': t.get('smoke_version_cmd', '-'),
            'smoke_help_cmd': t.get('smoke_help_cmd', '-'),
            'status': status_from_summary.get(tool, t.get('status', 'unknown')),
            'qa_rule': 'build+smoke required',
        }

lines = [
    '<!-- GENERATED FILE - DO NOT EDIT -->',
    '<!-- Regenerate with: scripts/tooling/generate-apptainer-qa-matrix.sh -->',
    '',
    '# APPTAINER_QA_MATRIX',
    '',
    '## Purpose',
    'Generated matrix for Apptainer-enabled tools and required QA gates.',
    '',
    '## Scope',
    'Derived from tool registries and container metadata fields.',
    '',
    '## Non-goals',
    '- Replacing full per-tool smoke manifests.',
    '',
    '## Contracts',
    '- Tool row exists iff registry runtimes include `apptainer`.',
    '- `apptainer_def` and smoke command fields are surfaced for QA checks.',
    '',
    '| Tool ID | Apptainer Def | Smoke Version | Smoke Help | QA Rule | Status |',
    '|---|---|---|---|---|---|',
]
for tool in sorted(rows):
    r = rows[tool]
    lines.append(
        f"| `{tool}` | `{r['apptainer_def']}` | `{r['smoke_version_cmd']}` | "
        f"`{r['smoke_help_cmd']}` | `{r['qa_rule']}` | `{r['status']}` |"
    )

out.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'generated {out}')
PY
