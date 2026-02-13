#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

usage() {
  cat <<'USAGE'
Usage: scripts/tooling/benchmarks.sh <fastq-stage|fastq-preprocess|fastq-all|fastq-status|bam-stage|bam-pipeline|bam-all>
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

./bin/require-isolate >/dev/null || {
  ./bin/require-isolate --explain >&2
  exit 1
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

mode="$1"
shift

run_id="${ISO_RUN_ID:-manual}"
out_dir="${OUT_DIR:-${ISO_ROOT:-$ROOT_DIR/artifacts}/benchmarks/${run_id}}"
tools="${TOOLS:-}"
allow_experimental="${ALLOW_EXPERIMENTAL:-0}"
sample_id="${SAMPLE_ID:-}"
r1="${R1:-}"
r2="${R2:-}"
bam="${BAM:-}"
bam_profile="${BAM_PROFILE:-bam-to-bam__default__v1}"
bam_stage="${BAM_STAGE:-validate}"
bam_sample_id="${BAM_SAMPLE_ID:-sample}"

if [[ "$out_dir" == *"/containers/smoke/"* || "$out_dir" == *"/containers/smoke" ]]; then
  echo "benchmark out dir must not overlap smoke logs: $out_dir" >&2
  exit 2
fi

bench_tools_args=()
if [[ -n "${tools}" ]]; then
  bench_tools_args=(--tools "${tools}")
fi

bench_experimental_args=()
if [[ "${allow_experimental}" == "1" || "${allow_experimental}" == "true" || "${allow_experimental}" == "yes" ]]; then
  bench_experimental_args=(--allow-experimental)
fi

if [[ -n "${BIJUX_BIN:-}" ]]; then
  read -r -a bijux_cmd <<< "${BIJUX_BIN}"
else
  bijux_cmd=("$ROOT_DIR/scripts/run.sh" tooling bijux)
fi

run_fastq_stage() {
  local stage="$1"
  if [[ -z "${stage}" || -z "${sample_id}" || -z "${r1}" ]]; then
    echo "ERROR: set STAGE=<trim|validate|...> SAMPLE_ID=<id> R1=<path>" >&2
    exit 2
  fi
  local cmd=("${bijux_cmd[@]}" bench fastq "${stage}" --sample-id "${sample_id}" --r1 "${r1}")
  if [[ -n "${r2}" ]]; then
    cmd+=(--r2 "${r2}")
  fi
  cmd+=(--out "${out_dir}")
  if [[ ${#bench_tools_args[@]} -gt 0 ]]; then cmd+=("${bench_tools_args[@]}"); fi
  if [[ ${#bench_experimental_args[@]} -gt 0 ]]; then cmd+=("${bench_experimental_args[@]}"); fi
  "${cmd[@]}"
}

run_bam_stage() {
  if [[ -z "${bam}" ]]; then
    echo "ERROR: set BAM=<path/to/input.bam>" >&2
    exit 2
  fi
  local cmd=("${bijux_cmd[@]}" bench bam stage --sample-id "${bam_sample_id}" --stage "${bam_stage}" --bam "${bam}" --out "${out_dir}")
  if [[ ${#bench_tools_args[@]} -gt 0 ]]; then cmd+=("${bench_tools_args[@]}"); fi
  "${cmd[@]}"
}

run_bam_pipeline() {
  if [[ -z "${bam}" ]]; then
    echo "ERROR: set BAM=<path/to/input.bam>" >&2
    exit 2
  fi
  local cmd=("${bijux_cmd[@]}" bench bam pipeline --sample-id "${bam_sample_id}" --profile "${bam_profile}" --bam "${bam}" --out "${out_dir}")
  if [[ ${#bench_tools_args[@]} -gt 0 ]]; then cmd+=("${bench_tools_args[@]}"); fi
  "${cmd[@]}"
}

case "${mode}" in
  fastq-stage)
    run_fastq_stage "${STAGE:-}"
    ;;
  fastq-preprocess)
    if [[ -z "${sample_id}" || -z "${r1}" ]]; then
      echo "ERROR: set SAMPLE_ID=<id> R1=<path>" >&2
      exit 2
    fi
    cmd=("${bijux_cmd[@]}" bench fastq preprocess --sample-id "${sample_id}" --r1 "${r1}" --out "${out_dir}")
    if [[ ${#bench_tools_args[@]} -gt 0 ]]; then cmd+=("${bench_tools_args[@]}"); fi
    if [[ ${#bench_experimental_args[@]} -gt 0 ]]; then cmd+=("${bench_experimental_args[@]}"); fi
    "${cmd[@]}"
    ;;
  fastq-all)
    run_fastq_stage validate
    run_fastq_stage trim
    run_fastq_stage filter
    run_fastq_stage stats
    run_fastq_stage qc-post
    run_fastq_stage screen
    "$0" fastq-preprocess
    if [[ -n "${r2}" ]]; then
      run_fastq_stage merge
      run_fastq_stage correct
      run_fastq_stage umi
    fi
    ;;
  fastq-status)
    "${bijux_cmd[@]}" bench status
    ;;
  bam-stage)
    run_bam_stage
    ;;
  bam-pipeline)
    run_bam_pipeline
    ;;
  bam-all)
    run_bam_stage
    run_bam_pipeline
    ;;
  *)
    echo "unsupported mode: ${mode}" >&2
    exit 2
    ;;
esac
