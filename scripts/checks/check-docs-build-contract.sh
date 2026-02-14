#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

CFG="$ROOT_DIR/configs/docs/mkdocs.toml"
[[ -f "$CFG" ]] || { echo "docs-build-contract: missing $CFG" >&2; exit 1; }

python3 - "$CFG" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

cfg = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
site_dir = str(cfg.get("site_dir", "")).strip()
if site_dir != "artifacts/docs/site":
    raise SystemExit(f"docs-build-contract: site_dir must be artifacts/docs/site (got {site_dir!r})")
print("docs-build-contract: cfg site_dir OK")
PY

if ! rg -q 'export XDG_CACHE_HOME="\$CACHE_DIR"' "$ROOT_DIR/scripts/tooling/docs-build.sh"; then
  echo "docs-build-contract: scripts/tooling/docs-build.sh must set XDG_CACHE_HOME to artifacts/docs/.cache" >&2
  exit 1
fi
if ! rg -q 'artifacts/docs/.cache' "$ROOT_DIR/scripts/tooling/docs-build.sh"; then
  echo "docs-build-contract: scripts/tooling/docs-build.sh must use artifacts/docs/.cache" >&2
  exit 1
fi
if ! rg -q 'PIP_CACHE_DIR="\$DOCS_CACHE"' "$ROOT_DIR/scripts/tooling/setup-docs-venv.sh"; then
  echo "docs-build-contract: scripts/tooling/setup-docs-venv.sh must set PIP_CACHE_DIR" >&2
  exit 1
fi
if ! rg -q 'artifacts/docs/.cache/pip' "$ROOT_DIR/scripts/tooling/setup-docs-venv.sh"; then
  echo "docs-build-contract: scripts/tooling/setup-docs-venv.sh must use artifacts/docs/.cache/pip" >&2
  exit 1
fi

echo "docs-build-contract: OK"
