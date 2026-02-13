#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
unset BASH_ENV ENV || true

dry_run=1
confirm=0
exclude_profile="push-default"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) dry_run=1 ;;
    --confirm) confirm=1; dry_run=0 ;;
    --exclude=*) exclude_profile="${1#*=}" ;;
    --exclude-profile=*) exclude_profile="${1#*=}" ;;
    --exclude) exclude_profile="${2:-}"; shift ;;
    --exclude-profile) exclude_profile="${2:-}"; shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

profiles_cfg="$ROOT_DIR/configs/hpc/lunarc_sync_profiles.toml"
exclude_file="$ROOT_DIR/configs/hpc/rsync/push-excludes.txt"
if [[ -f "$profiles_cfg" ]]; then
  found="$(python3 - <<'PY' "$profiles_cfg" "$exclude_profile"
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
cfg = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
name = sys.argv[2]
for p in cfg.get("profiles", []):
    if p.get("name") == name and p.get("exclude_file"):
        print(p["exclude_file"])
        break
PY
)"
  if [[ -n "$found" ]]; then
    exclude_file="$ROOT_DIR/$found"
  fi
fi

LUNARC_HOST="${LUNARC_HOST:-lunarc}"
LUNARC_ROOT="${LUNARC_ROOT:-${HOME}/bijux}"
LUNARC_REPO_DIR="${LUNARC_REPO_DIR:-${LUNARC_ROOT}/bijux-dna}"
CLEAN_CONTEXT="${CLEAN_CONTEXT:-1}"
ALLOW_DIRTY="${ALLOW_DIRTY:-0}"

ssh_clean() {
  local host="$1"
  local cmd="$2"
  ssh "$host" "$cmd" 2> >(grep -v -E '^bash: pyenv: command not found$' >&2)
}

rsync_clean() {
  rsync "$@" 2> >(grep -v -E '^bash: pyenv: command not found$' >&2)
}

if [[ "$ALLOW_DIRTY" != "1" ]]; then
  if ! git diff --quiet --ignore-submodules -- || ! git diff --cached --quiet --ignore-submodules --; then
    echo "refusing push: local git tree is dirty (set ALLOW_DIRTY=1 to override)" >&2
    exit 2
  fi
fi

if [[ "$dry_run" == "1" || "$confirm" != "1" ]]; then
  echo "[dry-run] would sync repo to $LUNARC_HOST:$LUNARC_REPO_DIR"
  echo "pass --confirm to execute"
  exit 0
fi

ssh_clean "$LUNARC_HOST" "mkdir -p '$LUNARC_REPO_DIR'"

if [[ "$CLEAN_CONTEXT" == "1" ]]; then
  temp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
  ensure_artifacts_dir "$temp_root"
  mkdir -p "$temp_root"
  files_from="$temp_root/tmp-lunarc-push-files.txt"
  trap 'rm -f "$files_from"' EXIT
  git ls-files >"$files_from"
  rsync_clean -az --delete --files-from="$files_from" ./ "$LUNARC_HOST:$LUNARC_REPO_DIR/"
else
  rsync_clean -az --delete \
    --exclude-from="$exclude_file" \
    ./ "$LUNARC_HOST:$LUNARC_REPO_DIR/"
fi

remote_commit="$(ssh_clean "$LUNARC_HOST" "cd '$LUNARC_REPO_DIR' && git rev-parse HEAD 2>/dev/null || echo 'no-git-repo'")"
remote_status="$(ssh_clean "$LUNARC_HOST" "cd '$LUNARC_REPO_DIR' && git status --short 2>/dev/null || true")"

echo "remote_repo=$LUNARC_REPO_DIR"
echo "remote_commit=$remote_commit"
if [[ -n "$remote_status" ]]; then
  echo "remote_status:"
  printf '%s\n' "$remote_status"
else
  echo "remote_status=clean"
fi
