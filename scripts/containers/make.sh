#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <subcommand>" >&2
  exit 2
fi

cmd="$1"
shift

container_type="${CONTAINER_TYPE:-docker-arm64}"
platform="${PLATFORM:-docker-arm64}"
tools="${TOOLS:-}"
stage="${STAGE:-}"
workers="${BIJUX_WORKERS:-1}"
container_artifact_dir="${CONTAINER_ARTIFACT_DIR:-artifacts/container}"
bijux_bin="${BIJUX_BIN:-./bin/isolate cargo run --bin bijux -- dna}"
bijux_hpc_root="${BIJUX_HPC_ROOT:-$HOME/bijux}"
domain="${DOMAIN:-}"
stages="${STAGES:-}"

read -r -a bijux_cmd <<< "${bijux_bin}"

check_container_type() {
  case "${container_type}" in
    docker-arm64|docker-amd64|apptainer) ;;
    *)
      echo "ERROR: unsupported CONTAINER_TYPE=${container_type}" >&2
      echo "supported: docker-arm64 | docker-amd64 | apptainer" >&2
      exit 2
      ;;
  esac
}

require_tools_or_stage() {
  if [[ -z "${tools}" && -z "${stage}" ]]; then
    echo "ERROR: set TOOLS=<tool_id> or STAGE=<stage>" >&2
    exit 2
  fi
}

smoke_apptainer_mode() {
  local mode="$1"
  local out="$2"
  ./bin/isolate env TOOLS="${tools}" BIJUX_WORKERS="${workers}" JOBS="${workers}" SMOKE_RUN_MODE="${mode}" SMOKE_LEVEL="contract" ARTIFACT_DIR="${out}" sh scripts/containers/smoke-apptainer.sh
}

case "${cmd}" in
  container-runtime-check)
    check_container_type
    echo "SYSTEM_TYPE=${SYSTEM_TYPE:-local} CONTAINER_TYPE=${container_type}"
    ;;
  env-prep)
    require_tools_or_stage
    if [[ -n "${stage}" ]]; then
      "${bijux_cmd[@]}" environment prep "${container_type}" --stage "${stage}"
    else
      "${bijux_cmd[@]}" environment prep "${container_type}" "${tools}"
    fi
    ;;
  env-smoke)
    require_tools_or_stage
    if [[ -n "${stage}" ]]; then
      "${bijux_cmd[@]}" environment smoke "${container_type}" --stage "${stage}"
    else
      "${bijux_cmd[@]}" environment smoke "${container_type}" "${tools}"
    fi
    ;;
  container-smoke)
    check_container_type
    require_tools_or_stage
    if [[ -n "${stage}" ]]; then
      "${bijux_cmd[@]}" environment prep "${container_type}" --stage "${stage}"
      "${bijux_cmd[@]}" environment smoke "${container_type}" --stage "${stage}"
    else
      "${bijux_cmd[@]}" environment prep "${container_type}" "${tools}"
      "${bijux_cmd[@]}" environment smoke "${container_type}" "${tools}"
    fi
    ;;
  containers-smoke)
    check_container_type
    while IFS= read -r st; do
      [[ -z "${st}" ]] && continue
      echo "== stage ${st}"
      "${bijux_cmd[@]}" environment prep "${container_type}" --stage "${st}"
      "${bijux_cmd[@]}" environment smoke "${container_type}" --stage "${st}"
    done < <("${bijux_cmd[@]}" registry list-stages)
    ;;
  smoke-containers-docker-arm64)
    ./bin/isolate env TOOLS="${tools}" BIJUX_WORKERS="${workers}" JOBS="${workers}" ARTIFACT_DIR="${container_artifact_dir}" sh scripts/containers/smoke-docker-arm64.sh
    ;;
  smoke-containers-docker-amd64)
    ./bin/isolate env TOOLS="${tools}" BIJUX_WORKERS="${workers}" JOBS="${workers}" ARTIFACT_DIR="${container_artifact_dir}" sh scripts/containers/smoke-docker-amd64.sh
    ;;
  smoke-containers-apptainer)
    ./bin/isolate env TOOLS="${tools}" BIJUX_WORKERS="${workers}" JOBS="${workers}" ARTIFACT_DIR="${container_artifact_dir}" sh scripts/containers/smoke-apptainer.sh
    ;;
  smoke-cntainers-apptainer-bijux-run)
    smoke_apptainer_mode "bijux-run" "${container_artifact_dir}/apptainer-bijux-run"
    ;;
  smoke-cntainers-apptainer-apptainer-run)
    smoke_apptainer_mode "apptainer-run" "${container_artifact_dir}/apptainer-apptainer-run"
    ;;
  smoke-cntainers-apptainer-verify)
    PYTHONPATH="scripts/tooling/python${PYTHONPATH:+:$PYTHONPATH}" python3 -m bijux_dna_tools.compare_apptainer_smoke "${container_artifact_dir}"
    ;;
  build-images)
    if [[ "${container_type}" != "docker-arm64" ]]; then
      echo "skip: build-images is docker-only (CONTAINER_TYPE=${container_type})"
      exit 0
    fi
    tools_val="${tools}"
    if [[ -z "${tools_val}" ]]; then
      tools_val="$("${bijux_cmd[@]}" registry list-tools --kind primary | paste -sd, -)"
    fi
    ./bin/isolate env TOOLS="${tools_val}" BIJUX_WORKERS="${workers}" JOBS="${workers}" SMOKE_LEVEL="build" SAVE_TAR="0" ARTIFACT_DIR="${container_artifact_dir}" sh scripts/containers/smoke-docker-arm64.sh
    ;;
  test-images)
    if [[ "${container_type}" == "docker-arm64" ]]; then
      if [[ -n "${stage}" ]]; then
        tools_val="$("${bijux_cmd[@]}" registry list-tools --stage "${stage}" --kind all | paste -sd, -)"
      else
        tools_val="${tools}"
        if [[ -z "${tools_val}" ]]; then
          tools_val="$("${bijux_cmd[@]}" registry list-tools --kind primary | paste -sd, -)"
        fi
      fi
      ./bin/isolate env TOOLS="${tools_val}" BIJUX_WORKERS="${workers}" JOBS="${workers}" SMOKE_LEVEL="contract" SAVE_TAR="0" ARTIFACT_DIR="${container_artifact_dir}" sh scripts/containers/smoke-docker-arm64.sh
    elif [[ -n "${stage}" ]]; then
      STAGE="${stage}" TOOLS="" "$0" env-smoke
    elif [[ -n "${tools}" ]]; then
      TOOLS="${tools}" STAGE="" "$0" env-smoke
    else
      "$0" containers-smoke
    fi
    ;;
  test-images-stage)
    if [[ -z "${stage}" ]]; then
      echo "ERROR: set STAGE=<domain.stage|stage> (example: STAGE=fastq.trim)" >&2
      exit 2
    fi
    TOOLS="" "$0" env-smoke
    ;;
  test-images-tool)
    if [[ -z "${tools}" ]]; then
      echo "ERROR: set TOOLS=<tool_id>" >&2
      exit 2
    fi
    STAGE="" "$0" env-smoke
    ;;
  image-smoke-vcf)
    tools_vcf="$(
      stages_vcf="$("${bijux_cmd[@]}" registry list-stages | awk -F. '$1=="vcf"{print $0}')"
      if [[ -z "${stages_vcf}" ]]; then
        echo ""
      else
        while IFS= read -r st; do
          [[ -z "${st}" ]] && continue
          "${bijux_cmd[@]}" registry list-tools --stage "${st}" --kind all
        done <<< "${stages_vcf}" | tr ',' '\n' | sed '/^$/d' | sort -u | paste -sd, -
      fi
    )"
    if [[ -z "${tools_vcf}" ]]; then
      echo "ERROR: no VCF tools found via registry stage/tool mapping" >&2
      exit 2
    fi
    if [[ "${container_type}" == "apptainer" ]]; then
      ./bin/isolate env TOOLS="${tools_vcf}" BIJUX_WORKERS="${workers}" JOBS="${workers}" ARTIFACT_DIR="${container_artifact_dir}" sh scripts/containers/smoke-apptainer.sh
    else
      ./bin/isolate env TOOLS="${tools_vcf}" BIJUX_WORKERS="${workers}" JOBS="${workers}" SMOKE_LEVEL="contract" SAVE_TAR="0" ARTIFACT_DIR="${container_artifact_dir}" sh scripts/containers/smoke-docker-arm64.sh
    fi
    ;;
  image-qa)
    if [[ "${container_type}" != "docker-arm64" ]]; then
      echo "skip: image-qa is docker-only (CONTAINER_TYPE=${container_type})"
      exit 0
    fi
    ./bin/isolate cargo run --bin image_qa -- --platform "${platform}"
    ;;
  apptainer-ensure)
    if [[ -z "${domain}" || -z "${stages}" ]]; then
      echo "ERROR: set DOMAIN=<domain> and STAGES=<comma-separated>" >&2
      echo "example: make apptainer-ensure DOMAIN=fastq STAGES=validate_pre,trim,filter,stats,qc_post" >&2
      exit 2
    fi
    BIJUX_HPC_ROOT="${bijux_hpc_root}" "${bijux_cmd[@]}" env ensure-images --domain "${domain}" --stages "${stages}"
    ;;
  apptainer-ensure-stage)
    if [[ -z "${domain}" || -z "${stages}" ]]; then
      echo "ERROR: set DOMAIN and STAGES for apptainer-ensure-stage" >&2
      exit 2
    fi
    BIJUX_HPC_ROOT="${bijux_hpc_root}" "${bijux_cmd[@]}" env ensure-images --domain "${domain}" --stages "${stages}"
    ;;
  *)
    echo "unsupported subcommand: ${cmd}" >&2
    exit 2
    ;;
esac
