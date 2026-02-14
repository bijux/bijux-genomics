#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'EOF'
Usage: scripts/domain/generate-index.sh <domain>|--all
EOF
}

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 2
fi

arg="$1"
domains=()
if [[ "$arg" == "--all" ]]; then
  while IFS= read -r d; do
    domains+=("$d")
  done < <(find "$ROOT_DIR/domain" -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | sort)
else
  domains=("$arg")
fi

python3 - "$ROOT_DIR" "${domains[@]}" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
domains = sys.argv[2:]


def parse_scalar(path: Path, key: str):
    text = path.read_text(encoding="utf-8")
    m = re.search(rf"^{re.escape(key)}:\s*\"?([^\"\n#]+)\"?\s*$", text, flags=re.MULTILINE)
    return m.group(1).strip() if m else None


def extract_block(text: str, key: str):
    lines = text.splitlines()
    start = None
    end = None
    for i, line in enumerate(lines):
        if re.match(rf"^{re.escape(key)}:\s*$", line):
            start = i
            continue
        if start is not None and re.match(r"^[A-Za-z0-9_]+:\s*", line):
            end = i
            break
    if start is None:
        return None, None
    if end is None:
        end = len(lines)
    return start, end


for dom in domains:
    dom_dir = root / "domain" / dom
    index_path = dom_dir / "index.yaml"
    if not index_path.exists():
        raise SystemExit(f"missing {index_path}")

    stage_ids = []
    for stage_file in sorted((dom_dir / "stages").glob("*.yaml")):
        if stage_file.name == "_schema.yaml":
            continue
        sid = parse_scalar(stage_file, "stage_id")
        if sid:
            stage_ids.append(sid)
    stage_ids = sorted(set(stage_ids))

    tool_ids = []
    for tool_file in sorted((dom_dir / "tools").glob("*.yaml")):
        if tool_file.name == "_schema.yaml":
            continue
        tid = parse_scalar(tool_file, "tool_id")
        if tid:
            tool_ids.append(tid)
    tool_ids = sorted(set(tool_ids))

    text = index_path.read_text(encoding="utf-8")
    lines = text.splitlines()
    # Strip previous generated header if present.
    if lines and lines[0].startswith("# GENERATED FILE - DO NOT EDIT"):
        lines = lines[1:]
        if lines and lines[0].startswith("# Regenerate with:"):
            lines = lines[1:]
        while lines and not lines[0].strip():
            lines = lines[1:]

    text = "\n".join(lines)
    s_start, s_end = extract_block(text, "stage_ids")
    t_start, t_end = extract_block(text, "tool_ids")
    if s_start is None or t_start is None:
        raise SystemExit(f"{index_path}: missing stage_ids/tool_ids blocks")

    out_lines = text.splitlines()
    stage_block = ["stage_ids:"] + [f"  - {x}" for x in stage_ids]
    tool_block = ["tool_ids:"] + [f"  - {x}" for x in tool_ids]

    # Replace from bottom to top so indices stay stable.
    if s_start > t_start:
        out_lines[t_start:t_end] = tool_block
        delta = len(tool_block) - (t_end - t_start)
        s_start += delta
        s_end += delta
        out_lines[s_start:s_end] = stage_block
    else:
        out_lines[s_start:s_end] = stage_block
        delta = len(stage_block) - (s_end - s_start)
        t_start += delta
        t_end += delta
        out_lines[t_start:t_end] = tool_block

    header = [
        "# GENERATED FILE - DO NOT EDIT",
        f"# Regenerate with: scripts/domain/generate-index.sh {dom}",
    ]
    final = "\n".join(header + [""] + out_lines) + "\n"
    index_path.write_text(final, encoding="utf-8")
    print(f"generated {index_path}")
PY
