#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" || "${1:-}" == "--dry-run" || "${1:-}" == "--verbose" ]]; then
  cat <<'USAGE'
Usage: scripts/checks/check-make-help-sync.sh
USAGE
  exit 0
fi

TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
readme_targets="$TMP_ROOT/make-help-readme.$$.txt"
help_targets="$TMP_ROOT/make-help-help.$$.txt"
trap 'rm -f "$readme_targets" "$help_targets"' EXIT

python3 - "$ROOT_DIR" > "$readme_targets" <<'PY'
from pathlib import Path
import sys
lines = (Path(sys.argv[1]) / "makefiles/README.md").read_text(encoding="utf-8").splitlines()
in_public = False
for line in lines:
    if line.strip() == "Public targets (stable contract):":
        in_public = True
        continue
    if in_public and line.startswith("- `"):
        print(line.split("`")[1])
        continue
    if in_public and line.strip() and not line.startswith("- "):
        break
PY

make help | awk '/^  [a-zA-Z0-9._-]+[[:space:]]/{print $1}' > "$help_targets"

if ! diff -u "$readme_targets" "$help_targets" >/dev/null; then
  echo "make-help-sync: README public targets and make help output drifted" >&2
  diff -u "$readme_targets" "$help_targets" >&2 || true
  exit 1
fi

echo "make-help-sync: OK"
