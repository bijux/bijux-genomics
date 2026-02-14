#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
cfg = root / "configs"
errors: list[str] = []

# 1) every configs subdir requires index.md
for d in sorted(p for p in cfg.rglob("*") if p.is_dir()):
    idx = d / "index.md"
    if not idx.exists():
        errors.append(f"{d.relative_to(root)}: missing index.md")
        continue
    # sibling discipline: index.md must reference all sibling config files in same directory.
    text = idx.read_text(encoding="utf-8")
    backticked = set(re.findall(r"`([^`]+)`", text))
    for sib in sorted(p for p in d.iterdir() if p.is_file() and p.name != "index.md"):
        rel = sib.relative_to(root).as_posix()
        if sib.suffix not in {".toml", ".yaml", ".yml", ".py", ".txt", ".md"}:
            continue
        if rel not in backticked and sib.name not in backticked:
            errors.append(f"{idx.relative_to(root)}: missing sibling reference `{sib.name}`")

# 2) every config file appears in exactly one index
cfg_files = [
    p for p in sorted(cfg.rglob("*"))
    if p.is_file() and p.suffix in {".toml", ".yaml", ".yml"}
]
index_files = sorted(cfg.rglob("index.md"))
mentions: dict[Path, list[Path]] = {p: [] for p in cfg_files}
for idx in index_files:
    text = idx.read_text(encoding="utf-8")
    backticked = set(re.findall(r"`([^`]+)`", text))
    for f in cfg_files:
        rel = f.relative_to(root).as_posix()
        base = f.name
        if rel in backticked:
            mentions[f].append(idx)
            continue
        if base in backticked and idx.parent == f.parent:
            mentions[f].append(idx)

for f, where in mentions.items():
    if len(where) != 1:
        places = ", ".join(str(p.relative_to(root)) for p in where) if where else "<none>"
        errors.append(f"{f.relative_to(root)}: expected exactly one index.md mention, found {len(where)} ({places})")

if errors:
    print("config-index-discipline: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("config-index-discipline: OK")
PY
