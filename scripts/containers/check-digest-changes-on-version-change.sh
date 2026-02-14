#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
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
in_ci = bool(__import__("os").environ.get("CI"))
head_versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
head_lock = json.loads((root / "containers/versions/lock.json").read_text(encoding="utf-8"))
head_digest = {str(i.get("tool")): str(i.get("resolved_image_digest", "")).strip() for i in head_lock.get("items", [])}

prev_rev = subprocess.run(["git", "-C", str(root), "rev-parse", "--verify", "HEAD^"], capture_output=True, text=True)
if prev_rev.returncode != 0:
    print("digest/version coupling: SKIP (no previous commit)")
    raise SystemExit(0)
prev_rev = prev_rev.stdout.strip()

def git_show(path: str) -> str:
    p = subprocess.run(["git", "-C", str(root), "show", f"{prev_rev}:{path}"], capture_output=True, text=True)
    if p.returncode != 0:
        return ""
    return p.stdout

prev_versions_txt = git_show("containers/versions/versions.toml")
prev_lock_txt = git_show("containers/versions/lock.json")
if not prev_versions_txt or not prev_lock_txt:
    print("digest/version coupling: SKIP (previous lock/version file missing)")
    raise SystemExit(0)

prev_versions = tomllib.loads(prev_versions_txt)
prev_lock = json.loads(prev_lock_txt)
prev_digest = {str(i.get("tool")): str(i.get("resolved_image_digest", "")).strip() for i in prev_lock.get("items", [])}

errors = []
for tool, row in head_versions.items():
    now_v = str((row or {}).get("version", "")).strip()
    prev_v = str((prev_versions.get(tool) or {}).get("version", "")).strip()
    if not prev_v or now_v == prev_v:
        continue
    d_prev = prev_digest.get(tool, "")
    d_now = head_digest.get(tool, "")
    if not d_now:
        if in_ci:
            errors.append(f"{tool}: version changed {prev_v} -> {now_v} but current lock digest is empty")
        continue
    elif d_prev and d_now == d_prev:
        errors.append(f"{tool}: version changed {prev_v} -> {now_v} but digest did not change ({d_now})")

if errors:
    print("digest/version coupling: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("digest/version coupling: OK")
PY
