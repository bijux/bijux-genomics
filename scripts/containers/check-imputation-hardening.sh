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
import re
import sys

root = Path(sys.argv[1])
tools = ["glimpse", "impute5", "minimac4", "shapeit5", "beagle", "eagle", "bcftools", "plink2"]
nonroot_ex = (root / "containers/docker/NONROOT_EXCEPTIONS.md").read_text(encoding="utf-8")
entrypoint_ex = (root / "containers/docker/ENTRYPOINT_EXCEPTIONS.md").read_text(encoding="utf-8")
wild_nonroot = "`*`" in nonroot_ex
wild_entrypoint = "`*`" in entrypoint_ex
errors = []

for t in tools:
    dockerfile = root / "containers/docker/arm64" / f"Dockerfile.{t}"
    if not dockerfile.exists():
        errors.append(f"{t}: missing dockerfile")
        continue
    text = dockerfile.read_text(encoding="utf-8")
    has_user = re.search(r"(?m)^USER\s+", text) is not None
    if not has_user and not wild_nonroot and f"`{t}`" not in nonroot_ex:
        errors.append(f"{t}: runs as root and is not listed in NONROOT_EXCEPTIONS.md")
    has_entrypoint = re.search(r"(?m)^ENTRYPOINT\s+\[", text) is not None
    has_cmd = re.search(r"(?m)^CMD\s+\[", text) is not None
    if (not has_entrypoint or not has_cmd) and not wild_entrypoint and f"`{t}`" not in entrypoint_ex:
        errors.append(f"{t}: missing JSON ENTRYPOINT/CMD and not listed in ENTRYPOINT_EXCEPTIONS.md")

if errors:
    print("imputation hardening policy: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("imputation hardening policy: OK")
PY
