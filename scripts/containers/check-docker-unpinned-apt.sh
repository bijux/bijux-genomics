#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" "${CI:-}" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
ci_mode = str(sys.argv[2]).strip().lower() in {"1", "true", "yes"}
errors = []

for dockerfile in sorted((root / "containers/docker/arm64").glob("Dockerfile.*")):
    lines = dockerfile.read_text(encoding="utf-8").splitlines()
    run_lines = [ln.strip() for ln in lines if "apt-get install" in ln or "apt install" in ln]
    for ln in run_lines:
        # Flatten escaped lines crudely; this check is intentionally conservative.
        segment = ln.split("apt-get install", 1)[-1] if "apt-get install" in ln else ln.split("apt install", 1)[-1]
        segment = re.sub(r"--[a-zA-Z0-9-]+(?:=[^\s]+)?", " ", segment)
        segment = segment.replace("&&", " ").replace("\\", " ")
        toks = [t for t in re.split(r"\s+", segment) if t and not t.startswith("-")]
        for tok in toks:
            if tok in {"install", "apt-get", "apt", "update", "rm", "-rf", "/var/lib/apt/lists/*"}:
                continue
            if tok.startswith("$") or tok.startswith("\"") or tok.startswith("/"):
                continue
            if tok in {";", "|"}:
                continue
            if "=" not in tok:
                errors.append(f"{dockerfile.relative_to(root)}: unpinned apt package '{tok}'")

if errors:
    if ci_mode:
        print("docker apt pin check: failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        raise SystemExit(1)
    print("docker apt pin check: WARN (non-CI mode)", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(0)

print("docker apt pin check: OK")
PY
