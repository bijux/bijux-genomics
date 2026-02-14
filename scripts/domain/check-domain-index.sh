#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"

status=0
for dom_dir in "$ROOT_DIR"/domain/*; do
  [[ -d "$dom_dir" ]] || continue
  dom="$(basename "$dom_dir")"
  idx="$dom_dir/index.yaml"
  [[ -f "$idx" ]] || continue
  if ! head -n 1 "$idx" | grep -q '^# GENERATED FILE - DO NOT EDIT$'; then
    echo "domain index: missing generated header in domain/$dom/index.yaml" >&2
    status=1
  fi
  if ! head -n 2 "$idx" | tail -n 1 | grep -q '^# Regenerate with: scripts/domain/generate-index.sh '; then
    echo "domain index: missing regenerate header in domain/$dom/index.yaml" >&2
    status=1
  fi
done

for dom_dir in "$ROOT_DIR"/domain/*; do
  [[ -d "$dom_dir" ]] || continue
  dom="$(basename "$dom_dir")"
  idx="$dom_dir/index.yaml"
  [[ -f "$idx" ]] || continue
  expected="$(mktemp "$TMP_ROOT/domain-index-${dom}.XXXXXX")"
  cp "$idx" "$expected"
  "$ROOT_DIR/scripts/domain/generate-index.sh" "$dom" >/dev/null
  if ! diff -u "$idx" "$expected" >/dev/null; then
    echo "domain index drift for domain/$dom/index.yaml; regenerate with scripts/domain/generate-index.sh $dom" >&2
    status=1
  fi
  mv "$expected" "$idx"
done

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
errors = []

def parse_list(text: str, key: str):
    out = []
    lines = text.splitlines()
    in_block = False
    for line in lines:
        if re.match(rf"^{re.escape(key)}:\s*$", line):
            in_block = True
            continue
        if in_block:
            m = re.match(r"^\s*-\s*([^\s#]+)\s*$", line)
            if m:
                out.append(m.group(1).strip().strip('"'))
                continue
            if line and not line.startswith(" "):
                break
    return out

def parse_stage_from_file(path: Path):
    m = re.search(r'^stage_id:\s*"?([^"\n#]+)"?\s*$', path.read_text(encoding="utf-8"), flags=re.MULTILINE)
    return m.group(1).strip() if m else None

for dom_dir in sorted((root / "domain").iterdir()):
    if not dom_dir.is_dir():
        continue
    index_path = dom_dir / "index.yaml"
    if not index_path.exists():
        continue
    idx_text = index_path.read_text(encoding="utf-8")
    stage_ids = parse_list(idx_text, "stage_ids")
    tool_ids = set(parse_list(idx_text, "tool_ids"))

    stage_file_map = {}
    for stage_file in sorted((dom_dir / "stages").glob("*.yaml")):
        if stage_file.name == "_schema.yaml":
            continue
        sid = parse_stage_from_file(stage_file)
        if sid:
            stage_file_map[sid] = stage_file
    for sid in stage_ids:
        if sid not in stage_file_map:
            errors.append(f"{index_path}: stage {sid} is listed but no stages/*.yaml declares it")
            continue
        fixture_dir = dom_dir / "fixtures" / sid
        if not fixture_dir.exists() or not any(p.is_file() for p in fixture_dir.rglob("*")):
            errors.append(f"{index_path}: stage {sid} must have at least one fixture under {fixture_dir.relative_to(root)}")

    tools_dir = dom_dir / "tools"
    declared_tools = set()
    for tool_file in tools_dir.glob("*.yaml"):
        if tool_file.name == "_schema.yaml":
            continue
        text = tool_file.read_text(encoding="utf-8")
        m = re.search(r'^tool_id:\s*"?([^"\n#]+)"?\s*$', text, flags=re.MULTILINE)
        if m:
            declared_tools.add(m.group(1).strip())
    for stage_dir in (dom_dir / "fixtures").glob("*"):
        if not stage_dir.is_dir():
            continue
        for tool_fixture in stage_dir.glob("*.txt"):
            tid = tool_fixture.stem
            if tid not in declared_tools:
                errors.append(
                    f"{tool_fixture.relative_to(root)}: fixture tool '{tid}' missing matching tools/<tool>.yaml in domain/{dom_dir.name}"
                )
    for tid in tool_ids:
        if tid not in declared_tools:
            errors.append(f"{index_path}: tool {tid} listed in tool_ids but missing tools/<tool>.yaml")

    # Reverse coverage: every stage/tool file must appear in index lists (no orphans).
    for sid in sorted(stage_file_map.keys()):
        if sid not in stage_ids:
            errors.append(f"{index_path}: missing stage_id listing for stages file declaring '{sid}'")
    for tid in sorted(declared_tools):
        if tid not in tool_ids:
            errors.append(f"{index_path}: missing tool_id listing for tools file declaring '{tid}'")

if errors:
    print("domain completeness check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("domain index/completeness: OK")
PY

if [[ "$status" -ne 0 ]]; then
  exit 1
fi
