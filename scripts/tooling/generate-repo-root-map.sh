#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/docs/00-intro/REPO_ROOT_MAP.generated.md}"

# Enforce script intent contracts before generating the map.
./scripts/run.sh checks tree-intent >/dev/null

python3 - <<'PY' "$ROOT_DIR" "$OUT"
from pathlib import Path
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])

# Parse configs/OWNERS.toml [[rule]] prefix->owner with minimal parser.
owners = []
owners_file = root / 'configs' / 'OWNERS.toml'
if owners_file.exists():
    cur = {}
    for raw in owners_file.read_text(encoding='utf-8').splitlines():
        s = raw.strip()
        if not s or s.startswith('#'):
            continue
        if s == '[[rule]]':
            if cur:
                owners.append(cur)
            cur = {}
            continue
        if '=' not in s:
            continue
        k, v = [x.strip() for x in s.split('=', 1)]
        cur[k] = v.strip('"')
    if cur:
        owners.append(cur)

def owner_for(rel: str) -> str:
    hits = [r['owner'] for r in owners if rel.startswith(r.get('prefix',''))]
    if len(hits) == 1:
        return hits[0]
    return '-'

rows = []
for p in sorted(root.iterdir()):
    if p.name.startswith('.'):
        continue
    rel = p.name
    kind = 'dir' if p.is_dir() else 'file'
    purpose = '-'
    readme = p / 'README.md' if p.is_dir() else None
    if readme and readme.exists():
        for line in readme.read_text(encoding='utf-8').splitlines():
            if line.startswith('Purpose:'):
                purpose = line.split(':', 1)[1].strip()
                break
    rows.append((rel, kind, owner_for(f'{rel}/' if kind=='dir' else rel), purpose))

script_rows = []
scripts_dir = root / "scripts"
if scripts_dir.exists():
    for d in sorted(p for p in scripts_dir.iterdir() if p.is_dir()):
        readme = d / "README.md"
        purpose = "-"
        if readme.exists():
            for line in readme.read_text(encoding="utf-8").splitlines():
                if line.startswith("Purpose:"):
                    purpose = line.split(":", 1)[1].strip()
                    break
        script_rows.append((d.relative_to(root).as_posix(), purpose))

lines = [
    '<!-- GENERATED FILE - DO NOT EDIT -->',
    '<!-- Regenerate with: scripts/tooling/generate-repo-root-map.sh -->',
    '',
    '# REPO_ROOT_MAP',
    '',
    '## Purpose',
    'Generated map of repository root entries with inferred ownership and intent.',
    '',
    '## Scope',
    'Top-level workspace paths only.',
    '',
    '## Non-goals',
    '- Replacing detailed per-subtree architecture docs.',
    '',
    '## Contracts',
    '- Ownership for config paths is sourced from `configs/OWNERS.toml`.',
    '- Script subtree intent is sourced from README `Purpose:` lines validated by `scripts/checks/tree-intent.sh`.',
    '',
    '| Path | Kind | Owner | Purpose |',
    '|---|---|---|---|',
]
for rel, kind, owner, purpose in rows:
    lines.append(f'| `{rel}` | `{kind}` | `{owner}` | {purpose} |')

lines.extend([
    '',
    '## Script Intent',
    '| Script Path | Purpose |',
    '|---|---|',
])
for rel, purpose in script_rows:
    lines.append(f'| `{rel}` | {purpose} |')

out.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'generated {out}')
PY
