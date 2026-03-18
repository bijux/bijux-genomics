#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

require_artifact_env

expected_artifact_root="${ROOT_DIR}/artifacts"
expected_target_dir="${expected_artifact_root}/target"
expected_cargo_home="${expected_artifact_root}/cargo/home"
expected_tmp_dir="${expected_artifact_root}/tmp"

[[ "${ARTIFACT_ROOT}" == "${expected_artifact_root}" ]] || {
  echo "artifact-env-contract: ARTIFACT_ROOT must resolve to ${expected_artifact_root}, got ${ARTIFACT_ROOT}" >&2
  exit 1
}
[[ "${ISO_ROOT}" == "${expected_artifact_root}" ]] || {
  echo "artifact-env-contract: ISO_ROOT compatibility alias must resolve to ${expected_artifact_root}, got ${ISO_ROOT}" >&2
  exit 1
}
[[ "${CARGO_TARGET_DIR}" == "${expected_target_dir}" ]] || {
  echo "artifact-env-contract: CARGO_TARGET_DIR must resolve to ${expected_target_dir}, got ${CARGO_TARGET_DIR}" >&2
  exit 1
}
[[ "${CARGO_HOME}" == "${expected_cargo_home}" ]] || {
  echo "artifact-env-contract: CARGO_HOME must resolve to ${expected_cargo_home}, got ${CARGO_HOME}" >&2
  exit 1
}
for path in "${TMPDIR}" "${TMP}" "${TEMP}"; do
  [[ "${path}" == "${expected_tmp_dir}" ]] || {
    echo "artifact-env-contract: temp path must resolve to ${expected_tmp_dir}, got ${path}" >&2
    exit 1
  }
done

if rg -n "/Users/|[A-Za-z]:\\\\Users\\\\" crates/*/tests/snapshots >/dev/null 2>&1; then
  echo "absolute host paths leaked into snapshots" >&2
  rg -n "/Users/|[A-Za-z]:\\\\Users\\\\" crates/*/tests/snapshots >&2 || true
  exit 1
fi

echo "artifact-env-contract: OK"
