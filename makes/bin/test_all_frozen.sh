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
status_file="${background_dir}/test-all.exit.status"
launcher_file="${background_dir}/test-all.launch.sh"
nextest_report="${rs_artifact_root}/test/${short_sha}/nextest-all.log"
artifact_target_dir="${artifact_root}/target"
artifact_cargo_home="${artifact_root}/cargo/home"
artifact_tmp_dir="${artifact_root}/tmp"
iso_root="${artifact_root}"

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is required but not installed" >&2
    exit 1
  fi
}

require_tool git
require_tool make
require_tool bash
require_tool python3

mkdir -p \
  "${worktree_root}" \
  "${background_dir}" \
  "${artifact_target_dir}" \
  "${artifact_cargo_home}" \
  "${artifact_tmp_dir}" \
  "$(dirname "${nextest_report}")"

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
artifact_target_dir=${artifact_target_dir}
artifact_cargo_home=${artifact_cargo_home}
artifact_tmp_dir=${artifact_tmp_dir}
rs_artifact_root=${rs_artifact_root}
console_log=${console_log}
nextest_report=${nextest_report}
status_file=${status_file}
launcher=${launcher_file}
EOF

rm -f "${status_file}"
cat >"${launcher_file}" <<EOF
#!/usr/bin/env bash
set -euo pipefail

cd "${worktree_dir}"

export ARTIFACT_ROOT="${artifact_root}"
export ARTIFACT_TARGET_DIR="${artifact_target_dir}"
export ARTIFACT_CARGO_HOME="${artifact_cargo_home}"
export ARTIFACT_TMP_DIR="${artifact_tmp_dir}"
export ISO_ROOT="${iso_root}"
export CARGO_TARGET_DIR="${artifact_target_dir}"
export CARGO_HOME="${artifact_cargo_home}"
export TMPDIR="${artifact_tmp_dir}"
export TMP="${artifact_tmp_dir}"
export TEMP="${artifact_tmp_dir}"
export RS_ARTIFACT_ROOT="${rs_artifact_root}"
export RS_RUN_ID="${short_sha}"

printf '%s\n' "frozen test-all start: ${short_sha}"
printf '%s\n' "worktree: ${worktree_dir}"
printf '%s\n' "artifact root: ${artifact_root}"
printf '%s\n' "cargo target dir: ${artifact_target_dir}"
printf '%s\n' "nextest report: ${nextest_report}"

status=0
if ! make test-all; then
  status=\$?
fi

printf '%s\n' "\${status}" >"${status_file}"
printf '%s\n' "frozen test-all exit: \${status}"
exit "\${status}"
EOF
chmod +x "${launcher_file}"

(
  cd "${worktree_dir}"
  background_pid="$(
    python3 - "${launcher_file}" "${console_log}" <<'PY'
import subprocess
import sys

launcher_path = sys.argv[1]
console_path = sys.argv[2]

with open(console_path, "wb", buffering=0) as console_file:
    process = subprocess.Popen(
        ["/bin/bash", launcher_path],
        stdin=subprocess.DEVNULL,
        stdout=console_file,
        stderr=subprocess.STDOUT,
        start_new_session=True,
        close_fds=True,
    )

print(process.pid)
PY
  )"
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
