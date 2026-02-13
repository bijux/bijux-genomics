#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT_ROOT="${1:-$ROOT_DIR/docs}"

./scripts/tooling/generate-tool-index.sh "$OUT_ROOT/20-science/TOOL_INDEX.md" >/dev/null
./scripts/containers/generate-qa-matrix.sh "$OUT_ROOT/30-operations/APPTAINER_QA_MATRIX.md" >/dev/null
./scripts/tooling/generate-repo-root-map.sh "$OUT_ROOT/00-intro/REPO_ROOT_MAP.generated.md" >/dev/null
./scripts/tooling/generate-compatibility-matrix.sh "$OUT_ROOT/50-reference/COMPATIBILITY_MATRIX.md" >/dev/null
./scripts/tooling/generate-docs-graph.sh "$OUT_ROOT/DOCS_GRAPH.toml" >/dev/null

echo "generated docs into $OUT_ROOT"
