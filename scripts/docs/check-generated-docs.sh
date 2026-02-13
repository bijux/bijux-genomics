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
tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
TMP_DIR="$(mktemp -d "$tmp_root/tmp-generated-docs.XXXXXX")"
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
check_header "$ROOT/docs/00-intro/REPO_ROOT_MAP.generated.md"
check_header "$ROOT/docs/50-reference/COMPATIBILITY_MATRIX.md"
while IFS= read -r g; do
  [[ -n "$g" ]] || continue
  check_header "$g"
done < <(find "$ROOT/docs" -type f -name '*.generated.md' | sort)

mkdir -p "$TMP_DIR/00-intro" "$TMP_DIR/20-science" "$TMP_DIR/30-operations" "$TMP_DIR/50-reference"
./scripts/tooling/generate-docs.sh "$TMP_DIR" >/dev/null

diff -u "$ROOT/docs/20-science/TOOL_INDEX.md" "$TMP_DIR/20-science/TOOL_INDEX.md" >/dev/null || {
  echo "generated-docs: docs/20-science/TOOL_INDEX.md drift; regenerate with scripts/tooling/generate-docs.sh" >&2
  exit 1
}
diff -u "$ROOT/docs/30-operations/APPTAINER_QA_MATRIX.md" "$TMP_DIR/30-operations/APPTAINER_QA_MATRIX.md" >/dev/null || {
  echo "generated-docs: docs/30-operations/APPTAINER_QA_MATRIX.md drift; regenerate with scripts/tooling/generate-docs.sh" >&2
  exit 1
}
diff -u "$ROOT/docs/00-intro/REPO_ROOT_MAP.generated.md" "$TMP_DIR/00-intro/REPO_ROOT_MAP.generated.md" >/dev/null || {
  echo "generated-docs: docs/00-intro/REPO_ROOT_MAP.generated.md drift; regenerate with scripts/tooling/generate-docs.sh" >&2
  exit 1
}
diff -u "$ROOT/docs/50-reference/COMPATIBILITY_MATRIX.md" "$TMP_DIR/50-reference/COMPATIBILITY_MATRIX.md" >/dev/null || {
  echo "generated-docs: docs/50-reference/COMPATIBILITY_MATRIX.md drift; regenerate with scripts/tooling/generate-docs.sh" >&2
  exit 1
}
diff -u "$ROOT/docs/DOCS_GRAPH.toml" "$TMP_DIR/DOCS_GRAPH.toml" >/dev/null || {
  echo "generated-docs: docs/DOCS_GRAPH.toml drift; regenerate with scripts/tooling/generate-docs.sh" >&2
  exit 1
}

echo "generated docs headers: OK"
