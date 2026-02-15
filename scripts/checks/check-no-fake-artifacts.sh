#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tmp_dir="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}/check-no-fake-artifacts"
ensure_artifacts_dir "$tmp_dir"
hits_file="$tmp_dir/hits.txt"
: > "$hits_file"

# 1) Executable stage code must not write placeholder bytes as artifacts.
rg -n -i \
  "atomic_write_bytes\([^)]*placeholder|write_bytes\([^)]*placeholder" \
  "$ROOT_DIR/crates/bijux-dna-stages-fastq/src" \
  "$ROOT_DIR/crates/bijux-dna-stages-bam/src" \
  "$ROOT_DIR/crates/bijux-dna-stages-vcf/src" \
  >>"$hits_file" || true

# 2) Produced artifact roots must not contain placeholder markers.
artifact_roots=(
  "$ROOT_DIR/artifacts/domain"
  "$ROOT_DIR/artifacts/containers/smoke"
  "$ROOT_DIR/artifacts/reports"
)
for artifact_root in "${artifact_roots[@]}"; do
  [[ -d "$artifact_root" ]] || continue
  rg -n -i --max-filesize 256K \
    --glob '**/*.json' \
    --glob '**/*.txt' \
    --glob '**/*.log' \
    --glob '**/*.md' \
    --glob '**/*.tsv' \
    --glob '**/*.yaml' \
    --glob '**/*.yml' \
    --glob '**/*.toml' \
    "placeholder|fake_artifact|dummy_artifact|stub_artifact" \
    "$artifact_root" \
    >>"$hits_file" || true
done

if [[ -s "$hits_file" ]]; then
  echo "no-fake-artifacts: FAIL"
  sed -n '1,120p' "$hits_file"
  exit 1
fi

echo "no-fake-artifacts: OK"
