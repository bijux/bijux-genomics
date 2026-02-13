#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

dry_run=1
confirm=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    --confirm) confirm=1; dry_run=0 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

LUNARC_HOST="${LUNARC_HOST:-lunarc}"
LUNARC_ROOT="${LUNARC_ROOT:-${HOME}/bijux}"
LUNARC_REPO_DIR="${LUNARC_REPO_DIR:-${LUNARC_ROOT}/bijux-dna}"
CLEAN_CONTEXT="${CLEAN_CONTEXT:-1}"
ALLOW_DIRTY="${ALLOW_DIRTY:-0}"

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

ssh "$LUNARC_HOST" "mkdir -p '$LUNARC_REPO_DIR'"

if [[ "$CLEAN_CONTEXT" == "1" ]]; then
  files_from="$(mktemp)"
  trap 'rm -f "$files_from"' EXIT
  git ls-files >"$files_from"
  rsync -az --delete --files-from="$files_from" ./ "$LUNARC_HOST:$LUNARC_REPO_DIR/"
else
  rsync -az --delete \
    --exclude-from="${SCRIPT_DIR}/rsync-push-excludes.txt" \
    ./ "$LUNARC_HOST:$LUNARC_REPO_DIR/"
fi

remote_commit="$(ssh "$LUNARC_HOST" "cd '$LUNARC_REPO_DIR' && git rev-parse HEAD 2>/dev/null || echo 'no-git-repo'")"
remote_status="$(ssh "$LUNARC_HOST" "cd '$LUNARC_REPO_DIR' && git status --short 2>/dev/null || true")"

echo "remote_repo=$LUNARC_REPO_DIR"
echo "remote_commit=$remote_commit"
if [[ -n "$remote_status" ]]; then
  echo "remote_status:"
  printf '%s\n' "$remote_status"
else
  echo "remote_status=clean"
fi
