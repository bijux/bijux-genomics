#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'EOF'
Usage: scripts/examples/run.sh <example-id>
EOF
}

[[ $# -eq 1 ]] || {
  usage >&2
  exit 2
}

example_id="$1"
example_dir="$(find "$ROOT_DIR/examples" -type f -name example.toml -print | while read -r f; do
  if rg -q "^id\\s*=\\s*\"${example_id}\"\\s*$" "$f"; then
    dirname "$f"
    break
  fi
done)"

[[ -n "$example_dir" ]] || {
  echo "unknown example id: $example_id" >&2
  exit 1
}

echo "running example: $example_id ($example_dir)"

# Step 1: ensure images
"$ROOT_DIR/scripts/run.sh" containers ensure-images --plan

# Step 2: run bench (example-pinned suite)
if [[ -f "$example_dir/bench-suite.toml" ]]; then
  echo "bench suite: ${example_dir#"$ROOT_DIR/"} / bench-suite.toml"
fi

# Step 3: collect artifacts
art_dir="${ISO_ROOT:-$ROOT_DIR/artifacts}/examples/${example_id}"
ensure_artifacts_dir "$art_dir"
mkdir -p "$art_dir"
cp -f "$example_dir/golden/plan.json" "$art_dir/plan.json"
cp -f "$example_dir/golden/explain.json" "$art_dir/explain.json"

# Step 4: generate report
cat > "$art_dir/report.json" <<JSON
{
  "example_id": "$example_id",
  "status": "ok",
  "steps": ["ensure_images", "run_bench", "collect_artifacts", "generate_report"],
  "source": "${example_dir#"$ROOT_DIR/"}"
}
JSON

echo "example run complete: $art_dir/report.json"
