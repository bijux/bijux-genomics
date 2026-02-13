#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ -z "${CI:-}" ]]; then
  echo "apptainer post pin policy: SKIP (CI-only gate)"
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
errors = []

for path in sorted((root / "containers/apptainer").rglob("*.def")):
    tool = path.stem
    text = path.read_text(encoding="utf-8", errors="ignore")
    post = ""
    if "%post" in text:
        post = text.split("%post", 1)[1].split("\n%", 1)[0]

    if not post.strip():
        errors.append(f"{path.relative_to(root)}: missing %post section")
        continue

    # No floating source refs in post
    if re.search(r"\b(latest|main|master|HEAD)\b", post, flags=re.I):
        errors.append(f"{path.relative_to(root)}: %post contains floating ref (latest/main/master/HEAD)")

    has_download = bool(re.search(r"\b(curl|wget)\b", post))
    if has_download:
        has_sha = ("sha256sum" in post) or ("shasum -a 256" in post)
        reg = versions.get(tool, {}) if isinstance(versions.get(tool), dict) else {}
        source_sha = str(reg.get("source_sha256", "")).strip()
        pin = str(reg.get("pinned_commit", "")).strip()
        if not has_sha:
            errors.append(f"{path.relative_to(root)}: %post downloads without checksum verification command")
        if not source_sha and not pin:
            errors.append(f"{path.relative_to(root)}: tool downloads in %post but versions.toml has neither source_sha256 nor pinned_commit")

if errors:
    print("apptainer post pin policy: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("apptainer post pin policy: OK")
PY
