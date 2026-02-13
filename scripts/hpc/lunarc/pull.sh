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
LUNARC_PULL_BASE="${LUNARC_PULL_BASE:-${HOME}/bijux}"
PULL_MODE="${PULL_MODE:-results}"
INCLUDE_CONTAINERS_MANIFEST="${INCLUDE_CONTAINERS_MANIFEST:-0}"
DATA_MANIFEST_GLOB="${DATA_MANIFEST_GLOB:-}"

ts="$(date +%Y%m%d-%H%M%S)"
dest="${LUNARC_PULL_BASE}/lunarc-${ts}"

if [[ "$dry_run" == "1" || "$confirm" != "1" ]]; then
  echo "[dry-run] would pull mode=$PULL_MODE from $LUNARC_HOST:$LUNARC_ROOT to $dest"
  echo "pass --confirm to execute"
  exit 0
fi

if [[ -e "$dest" ]]; then
  echo "refusing pull: destination already exists: $dest" >&2
  exit 2
fi
mkdir -p "$dest"

pulled_paths=()
if [[ "$PULL_MODE" == "full" ]]; then
  rsync -az \
    --exclude-from="${SCRIPT_DIR}/rsync-pull-full-excludes.txt" \
    "$LUNARC_HOST:$LUNARC_ROOT/" "$dest/"
  pulled_paths+=("$LUNARC_ROOT/")
else
  rsync -az \
    --include-from="${SCRIPT_DIR}/rsync-pull-results-includes.txt" \
    "$LUNARC_HOST:$LUNARC_ROOT/" "$dest/"
  pulled_paths+=("$LUNARC_ROOT/bijux-dna-results/")
  if [[ "$INCLUDE_CONTAINERS_MANIFEST" == "1" ]]; then
    mkdir -p "$dest/bijux-dna-containers"
    rsync -az "$LUNARC_HOST:$LUNARC_ROOT/bijux-dna-containers/manifest/" "$dest/bijux-dna-containers/manifest/" || true
    pulled_paths+=("$LUNARC_ROOT/bijux-dna-containers/manifest/")
  fi
  if [[ -n "$DATA_MANIFEST_GLOB" ]]; then
    IFS=',' read -r -a rels <<<"$DATA_MANIFEST_GLOB"
    for rel in "${rels[@]}"; do
      clean_rel="${rel#/}"
      mkdir -p "$(dirname "$dest/bijux-dna-data/$clean_rel")"
      rsync -az "$LUNARC_HOST:$LUNARC_ROOT/bijux-dna-data/$clean_rel" "$dest/bijux-dna-data/$clean_rel" || true
      pulled_paths+=("$LUNARC_ROOT/bijux-dna-data/$clean_rel")
    done
  fi
fi

remote_commit="$(ssh "$LUNARC_HOST" "cd '$LUNARC_REPO_DIR' && git rev-parse HEAD 2>/dev/null || echo 'no-git-repo'")"
remote_hostname="$(ssh "$LUNARC_HOST" "hostname -f 2>/dev/null || hostname")"
pulled_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
json_paths="$(printf '%s\n' "${pulled_paths[@]}" | sed '/^$/d' | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))')"

cat >"$dest/PULLED_FROM.json" <<JSON
{
  "schema_version": "bijux.lunarc.pull.v1",
  "remote_host": "${LUNARC_HOST}",
  "remote_hostname": "${remote_hostname}",
  "remote_root": "${LUNARC_ROOT}",
  "remote_repo": "${LUNARC_REPO_DIR}",
  "remote_commit": "${remote_commit}",
  "pulled_at_utc": "${pulled_at}",
  "pull_mode": "${PULL_MODE}",
  "paths": ${json_paths}
}
JSON

echo "pulled_to=$dest"
echo "meta=$dest/PULLED_FROM.json"
