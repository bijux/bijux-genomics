#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

OUT_BASE="${1:-${ROOT_DIR}/artifacts/test/fastq-gold-repro}"
RUN_A="${OUT_BASE}/run_a"
RUN_B="${OUT_BASE}/run_b"

rm -rf "${RUN_A}" "${RUN_B}"
mkdir -p "${RUN_A}" "${RUN_B}"

"${ROOT_DIR}/scripts/test/toy_runs.sh" run --profile fastq --out "${RUN_A}" >/dev/null
"${ROOT_DIR}/scripts/test/toy_runs.sh" run --profile fastq --out "${RUN_B}" >/dev/null

CHECKSUM_A="${RUN_A}/fastq_reference_adna/artifact_checksums.json"
CHECKSUM_B="${RUN_B}/fastq_reference_adna/artifact_checksums.json"
MANIFEST_A="${RUN_A}/fastq_reference_adna/manifest.json"
MANIFEST_B="${RUN_B}/fastq_reference_adna/manifest.json"
METRICS_A="${RUN_A}/fastq_reference_adna/metrics.json"
METRICS_B="${RUN_B}/fastq_reference_adna/metrics.json"

for p in "${CHECKSUM_A}" "${CHECKSUM_B}" "${MANIFEST_A}" "${MANIFEST_B}" "${METRICS_A}" "${METRICS_B}"; do
  [[ -f "${p}" ]] || {
    echo "fastq-gold-repro: missing expected artifact: ${p}" >&2
    exit 1
  }
done

diff -u "${CHECKSUM_A}" "${CHECKSUM_B}" >/dev/null || {
  echo "fastq-gold-repro: artifact checksum drift detected" >&2
  exit 1
}
diff -u "${MANIFEST_A}" "${MANIFEST_B}" >/dev/null || {
  echo "fastq-gold-repro: manifest drift detected" >&2
  exit 1
}
diff -u "${METRICS_A}" "${METRICS_B}" >/dev/null || {
  echo "fastq-gold-repro: metrics drift detected" >&2
  exit 1
}

echo "fastq-gold-repro: OK"
