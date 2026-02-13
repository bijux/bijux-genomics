#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
profiles_dir = root / "configs/runtime/profiles"
index_path = profiles_dir / "index.md"
readme_path = profiles_dir / "README.md"
errors = []

if not index_path.exists():
    errors.append("configs/runtime/profiles/index.md missing")
if not readme_path.exists():
    errors.append("configs/runtime/profiles/README.md missing")

index_text = index_path.read_text(encoding="utf-8") if index_path.exists() else ""
readme_text = readme_path.read_text(encoding="utf-8") if readme_path.exists() else ""

if "use-case" not in readme_text.lower() and "use case" not in readme_text.lower():
    errors.append("configs/runtime/profiles/README.md must document use-cases")

for toml in sorted(profiles_dir.glob("*.toml")):
    text = toml.read_text(encoding="utf-8")
    if not re.search(r'^use_case\s*=\s*".+?"\s*$', text, re.MULTILINE):
        errors.append(f"{toml.relative_to(root)} missing `use_case = \"...\"`")
    if toml.name not in index_text:
        errors.append(f"configs/runtime/profiles/index.md missing reference to {toml.name}")

if errors:
    print("runtime-profiles-contract: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("runtime-profiles-contract: OK")
PY
