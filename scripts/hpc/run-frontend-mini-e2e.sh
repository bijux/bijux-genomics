#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

RUN_ID="${ISO_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
OUT_DIR="${OUT_DIR:-${ISO_ROOT:-$ROOT_DIR/artifacts}/hpc/frontend-mini-e2e/$RUN_ID}"
mkdir -p "$OUT_DIR"
dry_run=1
confirm=0

while [[ $# -gt 0 ]]; do
  case "${1:-}" in
    --dry-run) dry_run=1; confirm=0; shift ;;
    --confirm) dry_run=0; confirm=1; shift ;;
    --help|-h)
      cat <<'USAGE'
Usage: scripts/hpc/run-frontend-mini-e2e.sh [--dry-run|--confirm]
USAGE
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ "$dry_run" -eq 1 ]]; then
  echo "[dry-run] run-frontend-mini-e2e (pass --confirm to execute)"
  exit 0
fi

REQUIRE_FRONTEND=1 "$SCRIPT_DIR/validate-frontend-constraints.sh" --confirm

run_one() {
  local example_id="$1"
  local label="$2"
  local start_ts end_ts
  local ex_out="$OUT_DIR/$label"
  mkdir -p "$ex_out"
  start_ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  set +e
  "$ROOT_DIR/scripts/examples/run.sh" --allow-non-isolate "$example_id" >"$ex_out/runner.stdout.log" 2>"$ex_out/runner.stderr.log"
  rc=$?
  set -e
  end_ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  src="${ISO_ROOT:-$ROOT_DIR/artifacts}/examples/$example_id"
  if [[ -d "$src" ]]; then
    cp -f "$src"/plan.json "$src"/explain.json "$src"/report.json "$src"/run_report.json "$src"/metrics.json "$src"/logs.txt "$ex_out/" 2>/dev/null || true
  fi

  domain_hash="$(shasum -a 256 "$ROOT_DIR/domain/$label/index.yaml" 2>/dev/null | awk '{print $1}')"
  example_toml="$(find "$ROOT_DIR/examples" -type f -name example.toml -print | while read -r f; do rg -q "^id\\s*=\\s*\"${example_id}\"\\s*$" "$f" && { echo "$f"; break; }; done)"
  config_hash="$(shasum -a 256 "$example_toml" 2>/dev/null | awk '{print $1}')"
  lock_sha="$(shasum -a 256 "$ROOT_DIR/containers/versions/lock.json" | awk '{print $1}')"
  write_json_sorted_file "$ex_out/frontend_run_meta.json" <<JSON
{
  "schema_version": "bijux.frontend.mini.e2e.v1",
  "example_id": "$example_id",
  "label": "$label",
  "start_utc": "$start_ts",
  "end_utc": "$end_ts",
  "exit_code": $rc,
  "host": "$(hostname -f 2>/dev/null || hostname)",
  "tool_versions_ref": "artifacts/containers/hpc/frontend-smoke/summary.json",
  "container_lock_sha256": "$lock_sha",
  "domain_hash_sha256": "$domain_hash",
  "config_hash_sha256": "$config_hash"
}
JSON
  return $rc
}

status=0
run_one "vcf_downstream_vcf_full_mini" "vcf" || status=1
run_one "fastq_edna_mini" "fastq" || status=1

write_json_sorted_file "$OUT_DIR/summary.json" <<JSON
{
  "schema_version": "bijux.frontend.mini.e2e.summary.v1",
  "run_id": "$RUN_ID",
  "out_dir": "$OUT_DIR",
  "status": $([[ "$status" -eq 0 ]] && echo "\"ok\"" || echo "\"fail\""),
  "examples": [
    {"id": "vcf_downstream_vcf_full_mini", "artifact_dir": "$OUT_DIR/vcf"},
    {"id": "fastq_edna_mini", "artifact_dir": "$OUT_DIR/fastq"}
  ]
}
JSON

echo "$OUT_DIR/summary.json"
exit "$status"
