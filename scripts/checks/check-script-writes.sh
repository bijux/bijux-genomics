#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"
failed=0

usage() {
  cat <<'USAGE'
Usage: scripts/checks/check-script-writes.sh
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ $# -gt 0 ]]; then
  echo "unknown argument: $1" >&2
  usage >&2
  exit 2
fi

parse_spec() {
  local spec_file="$1"
  python3 - "$spec_file" <<'PY'
import sys
from pathlib import Path
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

spec = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
for row in spec.get("script", []):
    path = row.get("path", "")
    outputs = row.get("outputs", [])
    print(f"{path}\t{','.join(outputs)}")
PY
}

while IFS=$'\t' read -r rel outputs; do
  [[ -n "$rel" ]] || continue
  file="$ROOT_DIR/$rel"
  [[ -f "$file" ]] || continue

  # Contract: each supported script must declare writable roots.
  if [[ -z "$outputs" ]]; then
    echo "check-script-writes: $rel has empty outputs contract in scripts/SUPPORTED.toml" >&2
    failed=1
  fi
  if ! grep -Eq '(^|,)artifacts/($|,)' <<<"$outputs"; then
    echo "check-script-writes: $rel outputs contract must include artifacts/" >&2
    failed=1
  fi
  if ! grep -Eq '(^|,)\$ISO_ROOT/($|,)' <<<"$outputs"; then
    echo "check-script-writes: $rel outputs contract must include \$ISO_ROOT/" >&2
    failed=1
  fi
  # Only approved output roots in SUPPORTED.toml outputs contract.
  IFS=',' read -r -a out_arr <<< "$outputs"
  for out in "${out_arr[@]}"; do
    o="$(echo "$out" | xargs)"
    case "$o" in
      "artifacts/"|"\$ISO_ROOT/"|"configs/vcf/panels/locks/"|"configs/vcf/deprecations/"|"configs/ci/registry/"|"containers/versions/"|"containers/docs/"|"docs/30-operations/"|"docs/50-reference/") ;;
      *)
        echo "check-script-writes: $rel has non-approved output root '$o' in scripts/SUPPORTED.toml" >&2
        failed=1
        ;;
    esac
  done

  # Static guard: ban obvious absolute writes in supported scripts.
  case "$rel" in
    scripts/checks/*|scripts/containers/check-*)
      ;;
    *)
      if rg -n '(>|>>|cp |mv |rm -rf|mkdir -p)\s*/(tmp|var|opt|usr|etc|home|Users)\b' "$file" >/dev/null 2>&1; then
        echo "check-script-writes: forbidden absolute write path pattern in $rel" >&2
        failed=1
      fi
      ;;
  esac
done < <(parse_spec "$SPEC")

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "check-script-writes: OK"
