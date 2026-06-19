#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
repo_name="$(basename "${repo_root}")"
frozen_ref="${TEST_ALL_FROZEN_REF:-HEAD}"
full_sha="$(git -C "${repo_root}" rev-parse "${frozen_ref}")"
short_sha="$(git -C "${repo_root}" rev-parse --short=9 "${full_sha}")"
workspace_root="$(cd "${repo_root}/.." && pwd)"
worktree_root="${TEST_ALL_FROZEN_WORKTREE_ROOT:-${workspace_root}/.bijux-worktrees/${repo_name}}"
worktree_dir="${worktree_root}/${short_sha}"
artifact_root="${repo_root}/artifacts/${short_sha}"
rs_artifact_root="${artifact_root}/rust"
background_dir="${artifact_root}/background"
console_log="${background_dir}/test-all.console.log"
pid_file="${background_dir}/test-all.pid"
meta_file="${background_dir}/test-all.meta"
nextest_report="${rs_artifact_root}/test/${short_sha}/nextest-all.log"

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is required but not installed" >&2
    exit 1
  fi
}

require_tool git
require_tool make
require_tool nohup

mkdir -p "${worktree_root}" "${background_dir}"

if [ -f "${pid_file}" ]; then
  existing_pid="$(cat "${pid_file}")"
  if [ -n "${existing_pid}" ] && kill -0 "${existing_pid}" 2>/dev/null; then
    echo "background test-all is already running for ${short_sha} (pid ${existing_pid})" >&2
    echo "console log: ${console_log}" >&2
    echo "nextest report: ${nextest_report}" >&2
    exit 1
  fi
fi

if [ -d "${worktree_dir}" ]; then
  if ! git -C "${worktree_dir}" rev-parse --show-toplevel >/dev/null 2>&1; then
    echo "existing path is not a git worktree: ${worktree_dir}" >&2
    exit 1
  fi

  worktree_sha="$(git -C "${worktree_dir}" rev-parse HEAD)"
  if [ "${worktree_sha}" != "${full_sha}" ]; then
    echo "existing worktree points to ${worktree_sha}, expected ${full_sha}: ${worktree_dir}" >&2
    exit 1
  fi

  if [ -n "$(git -C "${worktree_dir}" status --short)" ]; then
    echo "existing worktree is dirty: ${worktree_dir}" >&2
    exit 1
  fi
else
  git -C "${repo_root}" worktree add --detach "${worktree_dir}" "${full_sha}" >/dev/null
fi

cat >"${meta_file}" <<EOF
ref=${frozen_ref}
commit=${full_sha}
short_commit=${short_sha}
repo_root=${repo_root}
worktree=${worktree_dir}
artifact_root=${artifact_root}
rs_artifact_root=${rs_artifact_root}
console_log=${console_log}
nextest_report=${nextest_report}
EOF

(
  cd "${worktree_dir}"
  nohup env \
    ARTIFACT_ROOT="${artifact_root}" \
    RS_ARTIFACT_ROOT="${rs_artifact_root}" \
    RS_RUN_ID="${short_sha}" \
    make test-all >"${console_log}" 2>&1 &
  background_pid=$!
  printf '%s\n' "${background_pid}" >"${pid_file}"
)

background_pid="$(cat "${pid_file}")"
printf '%s\n' "started background test-all for ${short_sha}"
printf '%s\n' "ref: ${frozen_ref}"
printf '%s\n' "commit: ${full_sha}"
printf '%s\n' "worktree: ${worktree_dir}"
printf '%s\n' "artifact root: ${artifact_root}"
printf '%s\n' "console log: ${console_log}"
printf '%s\n' "nextest report: ${nextest_report}"
printf '%s\n' "pid: ${background_pid}"
