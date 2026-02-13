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
include_profile="pull-results-default"
exclude_profile="pull-full-default"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) dry_run=1 ;;
    --confirm) confirm=1; dry_run=0 ;;
    --include=*) include_profile="${1#*=}" ;;
    --include-profile=*) include_profile="${1#*=}" ;;
    --include) include_profile="${2:-}"; shift ;;
    --include-profile) include_profile="${2:-}"; shift ;;
    --exclude=*) exclude_profile="${1#*=}" ;;
    --exclude-profile=*) exclude_profile="${1#*=}" ;;
    --exclude) exclude_profile="${2:-}"; shift ;;
    --exclude-profile) exclude_profile="${2:-}"; shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

LUNARC_HOST="${LUNARC_HOST:-lunarc}"
LUNARC_ROOT="${LUNARC_ROOT:-${HOME}/bijux}"
LUNARC_REPO_DIR="${LUNARC_REPO_DIR:-${LUNARC_ROOT}/bijux-dna}"
LUNARC_PULL_BASE="${LUNARC_PULL_BASE:-${HOME}/bijux}"
PULL_MODE="${PULL_MODE:-results}"
INCLUDE_CONTAINERS_MANIFEST="${INCLUDE_CONTAINERS_MANIFEST:-0}"
DATA_MANIFEST_GLOB="${DATA_MANIFEST_GLOB:-}"

ssh_clean() {
  local host="$1"
  local cmd="$2"
  ssh "$host" "$cmd" 2> >(grep -v -E '^bash: pyenv: command not found$' >&2)
}

rsync_clean() {
  rsync "$@" 2> >(grep -v -E '^bash: pyenv: command not found$' >&2)
}
profiles_cfg="$ROOT_DIR/configs/hpc/lunarc_sync_profiles.toml"
pull_full_exclude="$ROOT_DIR/configs/hpc/rsync/pull-full-excludes.txt"
pull_results_include="$ROOT_DIR/configs/hpc/rsync/pull-results-includes.txt"
if [[ -f "$profiles_cfg" ]]; then
  full_found="$(python3 - <<'PY' "$profiles_cfg" "$exclude_profile"
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
  [[ -n "$full_found" ]] && pull_full_exclude="$ROOT_DIR/$full_found"
  res_found="$(python3 - <<'PY' "$profiles_cfg" "$include_profile"
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
cfg = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
name = sys.argv[2]
for p in cfg.get("profiles", []):
    if p.get("name") == name and p.get("include_file"):
        print(p["include_file"])
        break
PY
)"
  [[ -n "$res_found" ]] && pull_results_include="$ROOT_DIR/$res_found"
fi

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
  rsync_clean -az \
    --exclude-from="$pull_full_exclude" \
    "$LUNARC_HOST:$LUNARC_ROOT/" "$dest/"
  pulled_paths+=("$LUNARC_ROOT/")
else
  rsync_clean -az \
    --include-from="$pull_results_include" \
    "$LUNARC_HOST:$LUNARC_ROOT/" "$dest/"
  pulled_paths+=("$LUNARC_ROOT/bijux-dna-results/")
  if [[ "$INCLUDE_CONTAINERS_MANIFEST" == "1" ]]; then
    mkdir -p "$dest/bijux-dna-containers"
    rsync_clean -az "$LUNARC_HOST:$LUNARC_ROOT/bijux-dna-containers/manifest/" "$dest/bijux-dna-containers/manifest/" || true
    pulled_paths+=("$LUNARC_ROOT/bijux-dna-containers/manifest/")
  fi
  if [[ -n "$DATA_MANIFEST_GLOB" ]]; then
    IFS=',' read -r -a rels <<<"$DATA_MANIFEST_GLOB"
    for rel in "${rels[@]}"; do
      clean_rel="${rel#/}"
      mkdir -p "$(dirname "$dest/bijux-dna-data/$clean_rel")"
      rsync_clean -az "$LUNARC_HOST:$LUNARC_ROOT/bijux-dna-data/$clean_rel" "$dest/bijux-dna-data/$clean_rel" || true
      pulled_paths+=("$LUNARC_ROOT/bijux-dna-data/$clean_rel")
    done
  fi
fi

remote_commit="$(ssh_clean "$LUNARC_HOST" "cd '$LUNARC_REPO_DIR' && git rev-parse HEAD 2>/dev/null || echo 'no-git-repo'")"
remote_hostname="$(ssh_clean "$LUNARC_HOST" "hostname -f 2>/dev/null || hostname")"
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
