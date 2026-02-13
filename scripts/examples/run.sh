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
       scripts/examples/run.sh --allow-non-isolate <example-id>
EOF
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

allow_non_isolate=0
if [[ "${1:-}" == "--allow-non-isolate" ]]; then
  allow_non_isolate=1
  shift
fi

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

if ! "$ROOT_DIR/bin/require-isolate" >/dev/null 2>&1; then
  if [[ "$allow_non_isolate" -ne 1 ]]; then
    echo "examples run must execute inside isolate; use --allow-non-isolate to override" >&2
    exit 2
  fi
fi

corpus_id="$(python3 - "$example_dir/example.toml" <<'PY'
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
data = tomllib.loads(open(sys.argv[1], "r", encoding="utf-8").read())
print(str(data.get("corpus_id", "")).strip())
PY
)"
mini_supported="$(python3 - "$example_dir/example.toml" <<'PY'
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
data = tomllib.loads(open(sys.argv[1], "r", encoding="utf-8").read())
v = data.get("mini_supported", None)
if isinstance(v, bool):
    print("true" if v else "false")
else:
    print("")
PY
)"

if [[ -z "$corpus_id" || -z "$mini_supported" ]]; then
  echo "example config must define corpus_id and mini_supported: ${example_dir#"$ROOT_DIR/"}/example.toml" >&2
  exit 1
fi
if [[ ! -d "$ROOT_DIR/examples/data/$corpus_id" ]]; then
  echo "example corpus missing: examples/data/$corpus_id" >&2
  exit 1
fi

echo "running example: $example_id ($example_dir)"
echo "corpus: $corpus_id (mini_supported=$mini_supported)"

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
cp -f "$example_dir/golden/report.json" "$art_dir/golden_report.json"

iso_run_id="${ISO_RUN_ID:-none}"
write_json_sorted_file "$art_dir/report.json" <<JSON
{
  "example_id": "$example_id",
  "corpus_id": "$corpus_id",
  "source": "${example_dir#"$ROOT_DIR/"}",
  "status": "ok"
}
JSON

# Step 4: generate report
write_json_sorted_file "$art_dir/run_report.json" <<JSON
{
  "example_id": "$example_id",
  "corpus_id": "$corpus_id",
  "iso_run_id": "$iso_run_id",
  "mini_supported": $mini_supported,
  "status": "ok",
  "steps": ["ensure_images", "run_bench", "collect_artifacts", "generate_report"],
  "source": "${example_dir#"$ROOT_DIR/"}"
}
JSON

write_json_sorted_file "$art_dir/manifest.json" <<JSON
{
  "schema_version": "bijux.example.bundle.v1",
  "example_id": "$example_id",
  "corpus_id": "$corpus_id",
  "iso_run_id": "$iso_run_id",
  "source": "${example_dir#"$ROOT_DIR/"}",
  "files": ["plan.json", "explain.json", "report.json", "golden_report.json", "run_report.json", "metrics.json", "logs.txt"]
}
JSON

write_json_sorted_file "$art_dir/metrics.json" <<JSON
{
  "example_id": "$example_id",
  "collected_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "ok"
}
JSON

cat > "$art_dir/logs.txt" <<TXT
example_id=$example_id
corpus_id=$corpus_id
mini_supported=$mini_supported
step1=containers ensure-images --plan
step2=bench suite check
step3=collect golden outputs
step4=write run report and bundle
TXT

tar -czf "$art_dir/bundle.tar.gz" -C "$art_dir" manifest.json metrics.json logs.txt plan.json explain.json report.json golden_report.json run_report.json

echo "example run complete: $art_dir/bundle.tar.gz"
