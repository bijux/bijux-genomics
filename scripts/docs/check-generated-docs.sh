#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

check_header() {
  local file="$1"
  local head
  head=$(sed -n '1,6p' "$file")
  printf '%s\n' "$head" | grep -q '^<!-- GENERATED FILE - DO NOT EDIT -->$' || {
    echo "generated-docs: missing generated header in ${file#$ROOT/}" >&2
    return 1
  }
}

check_header "$ROOT/docs/30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md"
check_header "$ROOT/docs/20-science/TOOL_INDEX.md"
check_header "$ROOT/docs/30-operations/APPTAINER_QA_MATRIX.md"
check_header "$ROOT/docs/00-intro/REPO_ROOT_MAP.md"
check_header "$ROOT/docs/50-reference/COMPATIBILITY_MATRIX.md"

./scripts/tooling/generate-tool-index.sh "$TMP_DIR/TOOL_INDEX.md" >/dev/null
./scripts/tooling/generate-apptainer-qa-matrix.sh "$TMP_DIR/APPTAINER_QA_MATRIX.md" >/dev/null
./scripts/tooling/generate-repo-root-map.sh "$TMP_DIR/REPO_ROOT_MAP.md" >/dev/null
./scripts/tooling/generate-compatibility-matrix.sh "$TMP_DIR/COMPATIBILITY_MATRIX.md" >/dev/null

diff -u "$ROOT/docs/20-science/TOOL_INDEX.md" "$TMP_DIR/TOOL_INDEX.md" >/dev/null || {
  echo "generated-docs: docs/20-science/TOOL_INDEX.md drift; regenerate with scripts/tooling/generate-tool-index.sh" >&2
  exit 1
}
diff -u "$ROOT/docs/30-operations/APPTAINER_QA_MATRIX.md" "$TMP_DIR/APPTAINER_QA_MATRIX.md" >/dev/null || {
  echo "generated-docs: docs/30-operations/APPTAINER_QA_MATRIX.md drift; regenerate with scripts/tooling/generate-apptainer-qa-matrix.sh" >&2
  exit 1
}
diff -u "$ROOT/docs/00-intro/REPO_ROOT_MAP.md" "$TMP_DIR/REPO_ROOT_MAP.md" >/dev/null || {
  echo "generated-docs: docs/00-intro/REPO_ROOT_MAP.md drift; regenerate with scripts/tooling/generate-repo-root-map.sh" >&2
  exit 1
}
diff -u "$ROOT/docs/50-reference/COMPATIBILITY_MATRIX.md" "$TMP_DIR/COMPATIBILITY_MATRIX.md" >/dev/null || {
  echo "generated-docs: docs/50-reference/COMPATIBILITY_MATRIX.md drift; regenerate with scripts/tooling/generate-compatibility-matrix.sh" >&2
  exit 1
}

echo "generated docs headers: OK"
