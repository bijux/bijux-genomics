#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
ref = root / "assets" / "reference"
if not ref.exists():
    print("assets-reference-schema: assets/reference missing", file=sys.stderr)
    sys.exit(1)

errors = []

# 1) required keys + duplicate IDs in each yaml
for y in sorted(list(ref.rglob("*.yaml")) + list(ref.rglob("*.yml"))):
    text = y.read_text(encoding="utf-8")
    rel = y.relative_to(root).as_posix()
    if not re.search(r"^schema_version:\s*\S+", text, re.M):
        errors.append(f"{rel}: missing schema_version")

    non_comment_keys = [
        ln for ln in text.splitlines()
        if ln.strip() and not ln.strip().startswith("#") and ":" in ln
    ]
    if len(non_comment_keys) < 2:
        errors.append(f"{rel}: expected schema_version plus at least one additional key")

    ids = []
    for m in re.finditer(r"^\s*-\s*id:\s*([A-Za-z0-9_.-]+)\s*$", text, re.M):
        ids.append(m.group(1))
    dups = sorted({i for i in ids if ids.count(i) > 1})
    if dups:
        errors.append(f"{rel}: duplicated ids: {', '.join(dups)}")

# 2) preset reference resolvability per domain dir (where bank + presets co-exist)
for d in sorted(p for p in ref.iterdir() if p.is_dir()):
    bank_files = [
        p for p in sorted(list(d.glob("*.yaml")) + list(d.glob("*.yml")))
        if "presets" not in p.name
    ]
    preset_files = sorted(d.glob("*presets*.yaml")) + sorted(d.glob("*presets*.yml"))
    if not preset_files:
        continue

    bank_ids = set()
    for bf in bank_files:
        txt = bf.read_text(encoding="utf-8")
        for m in re.finditer(r"^\s*-\s*id:\s*([A-Za-z0-9_.-]+)\s*$", txt, re.M):
            bank_ids.add(m.group(1))

    for pf in preset_files:
        rel = pf.relative_to(root).as_posix()
        txt = pf.read_text(encoding="utf-8")
        # Validate any <something>_ids lists.
        for block in re.finditer(r"^\s*[A-Za-z0-9_]+_ids:\s*$", txt, re.M):
            start = block.end()
            after = txt[start:]
            vals = []
            for line in after.splitlines():
                if not line.startswith("      - ") and not line.startswith("    - "):
                    if line.strip() == "":
                        continue
                    if re.match(r"^\s*[A-Za-z0-9_]+:\s*", line):
                        break
                m = re.match(r"^\s*-\s*([A-Za-z0-9_.-]+)\s*$", line)
                if m:
                    vals.append(m.group(1))
            for v in vals:
                if bank_ids and v not in bank_ids:
                    errors.append(f"{rel}: unresolved preset reference id: {v}")

if errors:
    print("assets-reference-schema: FAILED", file=sys.stderr)
    for e in errors:
        print(f"  - {e}", file=sys.stderr)
    sys.exit(1)

print("assets-reference-schema: OK")
PY
