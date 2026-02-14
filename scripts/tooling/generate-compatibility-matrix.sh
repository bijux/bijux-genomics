#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/docs/50-reference/COMPATIBILITY_MATRIX.md}"
CATALOG="$ROOT_DIR/crates/bijux-dna-core/src/id_catalog.rs"
REGS=(
  "$ROOT_DIR/configs/ci/registry/tool_registry.toml"
  "$ROOT_DIR/configs/ci/registry/tool_registry_vcf.toml"
  "$ROOT_DIR/configs/ci/registry/tool_registry_experimental.toml"
  "$ROOT_DIR/configs/ci/registry/tool_registry_vcf_downstream.toml"
)

python3 - "$OUT" "$CATALOG" "${REGS[@]}" <<'PY'
from pathlib import Path
import re
import sys

out = Path(sys.argv[1])
catalog = Path(sys.argv[2])
regs = [Path(p) for p in sys.argv[3:]]

profiles = []
for line in catalog.read_text(encoding='utf-8').splitlines():
    m = re.match(r'pub const PIPELINE_([A-Z0-9_]+): &str = "([^"]+)";', line.strip())
    if m:
        profiles.append((m.group(1), m.group(2)))

tool_count = 0
for rp in regs:
    if not rp.exists():
        continue
    for line in rp.read_text(encoding='utf-8').splitlines():
        if line.strip().startswith('[[tools]]'):
            tool_count += 1

rows = []
for key, pid in profiles:
    domain = pid.split('-to-', 1)[0]
    stability = 'stable' if 'reference' in pid or 'default' in pid else 'experimental'
    rows.append((pid, domain, stability, 'v1', 'v1', 'compatible if stage/tool contracts unchanged'))

lines = [
    '<!-- GENERATED FILE - DO NOT EDIT -->',
    '<!-- Regenerate with: scripts/tooling/generate-compatibility-matrix.sh -->',
    '',
    '# COMPATIBILITY_MATRIX',
    '',
    '## Purpose',
    'Generated compatibility matrix derived from pipeline profile IDs and tool registry inventory.',
    '',
    '## Scope',
    f'Profiles sourced from `crates/bijux-dna-core/src/id_catalog.rs`; registries include {tool_count} tool entries.',
    '',
    '## Non-goals',
    '- Replacing detailed per-domain migration guides.',
    '',
    '## Contracts',
    '- Matrix is generated-only and must not be manually edited.',
    '- Breaking contract changes require version/schema updates and matrix regeneration.',
    '',
    '| Pipeline Profile | Domain | Stability | Plan Contract | Report Contract | Compatibility Rule |',
    '|---|---|---|---|---|---|',
]
for row in sorted(rows):
    lines.append(f'| `{row[0]}` | `{row[1]}` | `{row[2]}` | `{row[3]}` | `{row[4]}` | {row[5]} |')

out.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'generated {out}')
PY
