#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import json
import subprocess
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
policy_path = root / "configs/ci/tools/image_size_policy.toml"
lock_path = root / "containers/versions/lock.json"
if not policy_path.exists() or not lock_path.exists():
    print("image size regression: SKIP (missing policy/lock)")
    raise SystemExit(0)

policy = tomllib.loads(policy_path.read_text(encoding="utf-8"))
default_limit = float(policy.get("max_growth_percent_for_promoted", 20))
acks = {}
for row in policy.get("acknowledgement", []):
    tool = str(row.get("tool_id", "")).strip()
    frm = str(row.get("from_version", "")).strip()
    to = str(row.get("to_version", "")).strip()
    limit = float(row.get("max_growth_percent", default_limit))
    if tool and frm and to:
        acks[(tool, frm, to)] = limit

current = json.loads(lock_path.read_text(encoding="utf-8"))
current_items = {str(i.get("tool", "")): i for i in current.get("items", [])}

prev_text = ""
for rev in ("HEAD~1", "HEAD"):
    proc = subprocess.run(
        ["git", "-C", str(root), "show", f"{rev}:containers/versions/lock.json"],
        capture_output=True, text=True, check=False
    )
    if proc.returncode == 0 and proc.stdout.strip():
        if rev == "HEAD":
            continue
        prev_text = proc.stdout
        break
if not prev_text.strip():
    print("image size regression: SKIP (no previous lock available)")
    raise SystemExit(0)

prev = json.loads(prev_text)
prev_items = {str(i.get("tool", "")): i for i in prev.get("items", [])}

errors = []
checked = 0
for tool, cur in sorted(current_items.items()):
    if str(cur.get("status", "")).strip() != "production":
        continue
    old = prev_items.get(tool)
    if not old:
        continue
    old_size = int(old.get("image_size_bytes", 0) or 0)
    new_size = int(cur.get("image_size_bytes", 0) or 0)
    if old_size <= 0 or new_size <= 0:
        continue
    checked += 1
    growth = ((new_size - old_size) / old_size) * 100.0
    frm = str(old.get("version", "")).strip()
    to = str(cur.get("version", "")).strip()
    limit = acks.get((tool, frm, to), default_limit)
    if growth > limit:
        errors.append(
            f"{tool}: image grew {growth:.2f}% ({old_size} -> {new_size}) over allowed {limit:.2f}% "
            f"(version {frm} -> {to}); add acknowledgement if intentional"
        )

if errors:
    print("image size regression: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print(f"image size regression: OK ({checked} promoted tools compared)")
PY

