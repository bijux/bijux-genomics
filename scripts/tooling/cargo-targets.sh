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

run_in_artifact_env() {
  require_artifact_env
  bash -ceu "$1" bash "$@"
}

common_test_env='export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="${CARGO_TARGET_DIR}"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$(command -v sccache)"; fi;'

case "${cmd}" in
  policy-fast)
    require_artifact_env
    cargo test -p bijux-dna-policies --test dependency_graph --test purity_scans --test core_layering --test domain_dependency_policy --test ci_tools_policy --test dev_deps_policy --test heavy_deps_policy
    ;;
  ssot-policy-fast)
    run_in_artifact_env "${common_test_env} cargo test -p bijux-dna-policies --test contracts policy_test_names_are_consistent -- --nocapture; cargo test -p bijux-dna-policies --test contracts supported_stages_and_tools_are_complete -- --nocapture; cargo test -p bijux-dna-policies --test contracts each_tool_has_exactly_one_domain_and_stage_binding -- --nocapture"
    ;;
  test-profile-invariants)
    run_in_artifact_env "${common_test_env} cargo test -p bijux-dna-pipelines --test invariant_fast -- --nocapture"
    ;;
  registry-lint)
    run_in_artifact_env "${common_test_env} cargo test -p bijux-dna-policies --test contracts production_registry_is_pinned_and_non_floating -- --nocapture; cargo test -p bijux-dna-policies --test contracts profiles_only_use_valid_production_tools -- --nocapture"
    ;;
  unit-contract-fast)
    run_in_artifact_env "${common_test_env} cargo test -p bijux-dna-runner --lib -- --nocapture; cargo test -p bijux-dna-planner-fastq --lib -- --nocapture; cargo test -p bijux-dna-planner-bam --lib -- --nocapture; cargo test -p bijux-dna-stages-fastq --lib -- --nocapture; cargo test -p bijux-dna-stages-bam --lib -- --nocapture; cargo test -p bijux-dna-api --lib -- --nocapture"
    ;;
  release-readiness)
    run_in_artifact_env "${common_test_env} cargo test -p bijux-dna-policies --test contracts profiles_release_readiness_gate -- --nocapture; cargo test -p bijux-dna-policies --test contracts reference_adna_profile_uses_production_tools_only -- --nocapture"
    ;;
  policy-full)
    require_artifact_env
    cargo test -p bijux-dna-policies
    ;;
  domain-coverage)
    require_artifact_env
    cargo run -p bijux-dna --bin bijux-dna -- domain coverage --domain-dir domain
    ;;
  snapshots)
    require_artifact_env
    cargo insta test --workspace
    ;;
  snapshots-accept)
    require_artifact_env
    cargo insta accept --workspace
    ;;
  snapshots-review)
    require_artifact_env
    cargo insta review
    ;;
  fix-snapshots)
    require_artifact_env
    cargo insta test --workspace
    cargo insta accept --workspace
    ;;
  policy-only-fast-gate)
    run_in_artifact_env 'export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="${CARGO_TARGET_DIR}"; cargo test -p bijux-dna-policies --test contracts --test boundaries --test determinism -- --nocapture; cargo test -p bijux-dna-core --test contracts -- --nocapture; cargo test -p bijux-dna-pipelines --test contracts -- --nocapture; cargo test -p bijux-dna-runtime --test contracts -- --nocapture'
    ;;
  vcf-certification)
    run_in_artifact_env "${common_test_env} cargo nextest run -p bijux-dna-stages-vcf --all-features --failure-output immediate-final --no-tests pass"
    ;;
  ci-clippy-executors)
    ./scripts/tooling/ci-clippy-executors.sh
    ;;
  nextest-run)
    run_in_artifact_env "${common_test_env} cargo nextest run $*"
    ;;
  bam-smoke-test)
    run_in_artifact_env "${common_test_env} cargo test -p bijux-dna-api bam_smoke_runner_minimal_pipeline_validates_report_section_presence -- --exact"
    ;;
  *)
    echo "unsupported subcommand: ${cmd}" >&2
    exit 2
    ;;
esac
