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

run_in_isolate() {
  ./bin/isolate sh -ceu "$1"
}

common_test_env='export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$(command -v sccache)"; fi;'

case "${cmd}" in
  policy-fast)
    ./bin/isolate cargo test -p bijux-dna-policies --test dependency_graph --test purity_scans --test core_layering --test domain_dependency_policy --test ci_tools_policy --test dev_deps_policy --test heavy_deps_policy
    ;;
  ssot-policy-fast)
    run_in_isolate "./bin/require-isolate >/dev/null; ${common_test_env} cargo test -p bijux-dna-policies --test contracts policy_test_names_are_consistent -- --nocapture; cargo test -p bijux-dna-policies --test contracts supported_stages_and_tools_are_complete -- --nocapture; cargo test -p bijux-dna-policies --test contracts each_tool_has_exactly_one_domain_and_stage_binding -- --nocapture"
    ;;
  test-profile-invariants)
    run_in_isolate "./bin/require-isolate >/dev/null; ${common_test_env} cargo test -p bijux-dna-pipelines --test invariant_fast -- --nocapture"
    ;;
  registry-lint)
    run_in_isolate "./bin/require-isolate >/dev/null; ${common_test_env} cargo test -p bijux-dna-policies --test contracts production_registry_is_pinned_and_non_floating -- --nocapture; cargo test -p bijux-dna-policies --test contracts profiles_only_use_valid_production_tools -- --nocapture"
    ;;
  unit-contract-fast)
    run_in_isolate "./bin/require-isolate >/dev/null; ${common_test_env} cargo test -p bijux-dna-runner --lib -- --nocapture; cargo test -p bijux-dna-planner-fastq --lib -- --nocapture; cargo test -p bijux-dna-planner-bam --lib -- --nocapture; cargo test -p bijux-dna-stages-fastq --lib -- --nocapture; cargo test -p bijux-dna-stages-bam --lib -- --nocapture; cargo test -p bijux-dna-api --lib -- --nocapture"
    ;;
  release-readiness)
    run_in_isolate "./bin/require-isolate >/dev/null; ${common_test_env} cargo test -p bijux-dna-policies --test contracts profiles_release_readiness_gate -- --nocapture; cargo test -p bijux-dna-policies --test contracts reference_adna_profile_uses_production_tools_only -- --nocapture"
    ;;
  policy-full)
    ./bin/isolate cargo test -p bijux-dna-policies
    ;;
  domain-coverage)
    ./bin/isolate cargo run -p bijux-dna --bin bijux -- dna domain coverage --domain-dir domain
    ;;
  snapshots)
    ./bin/isolate cargo insta test --workspace
    ;;
  snapshots-accept)
    ./bin/isolate cargo insta accept --workspace
    ;;
  snapshots-review)
    ./bin/isolate cargo insta review
    ;;
  fix-snapshots)
    ./bin/isolate cargo insta test --workspace
    ./bin/isolate cargo insta accept --workspace
    ;;
  policy-only-fast-gate)
    run_in_isolate "./bin/require-isolate >/dev/null; export TZ=UTC LC_ALL=C CARGO_TARGET_DIR=\"\$ISO_ROOT/target-test\"; cargo test -p bijux-dna-policies --test contracts --test boundaries --test determinism -- --nocapture; cargo test -p bijux-dna-core --test contracts -- --nocapture; cargo test -p bijux-dna-pipelines --test contracts -- --nocapture; cargo test -p bijux-dna-runtime --test contracts -- --nocapture"
    ;;
  *)
    echo "unsupported subcommand: ${cmd}" >&2
    exit 2
    ;;
esac
