#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

mode="${1:-}"
if [[ -z "$mode" ]]; then
  echo "Usage: $0 <build|lint|serve>" >&2
  exit 2
fi

CFG="${DOCS_CFG:-$ROOT_DIR/configs/docs/mkdocs.toml}"
DOCS_VENV="${DOCS_VENV:-$ROOT_DIR/artifacts/docs/.venv}"
mkdocs_bin="${DOCS_VENV}/bin/mkdocs"
CACHE_DIR="${ROOT_DIR}/artifacts/docs/.cache"

require_file "$CFG"
require_file "$mkdocs_bin"

read_cfg() {
  python3 - <<'PY' "$CFG"
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib
    except ModuleNotFoundError:
        import toml as tomllib

cfg = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(cfg.get("mkdocs_config", "mkdocs.yml"))
print(cfg.get("site_dir", "artifacts/docs/site"))
print("true" if bool(cfg.get("strict", True)) else "false")
print(cfg.get("dev_addr", "127.0.0.1:8000"))
PY
}

cfg_text="$(read_cfg)"
mkdocs_config="$(printf '%s\n' "$cfg_text" | sed -n '1p')"
site_dir="$(printf '%s\n' "$cfg_text" | sed -n '2p')"
strict="$(printf '%s\n' "$cfg_text" | sed -n '3p')"
dev_addr="$(printf '%s\n' "$cfg_text" | sed -n '4p')"

if [[ "$site_dir" != "artifacts/docs/site" ]]; then
  echo "docs-build: site_dir must be artifacts/docs/site (got: $site_dir)" >&2
  exit 1
fi

mkdir -p "$CACHE_DIR"
export XDG_CACHE_HOME="$CACHE_DIR"

case "$mode" in
  build)
    "$mkdocs_bin" build --config-file "$ROOT_DIR/$mkdocs_config" --site-dir "$ROOT_DIR/$site_dir"
    ;;
  lint)
    if [[ "$strict" == "true" ]]; then
      "$mkdocs_bin" build --strict --config-file "$ROOT_DIR/$mkdocs_config" --site-dir "$ROOT_DIR/$site_dir"
    else
      "$mkdocs_bin" build --config-file "$ROOT_DIR/$mkdocs_config" --site-dir "$ROOT_DIR/$site_dir"
    fi
    ;;
  serve)
    "$mkdocs_bin" serve --config-file "$ROOT_DIR/$mkdocs_config" --dev-addr "$dev_addr"
    ;;
  *)
    echo "Usage: $0 <build|lint|serve>" >&2
    exit 2
    ;;
esac
