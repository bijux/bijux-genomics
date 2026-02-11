#!/usr/bin/env bash
set -euo pipefail
export TZ=UTC
export LC_ALL=C

# Batch-build Apptainer defs in a VM-local writable directory.
# Sequential by default; optional limited concurrency.

DEFS_DIR="containers/apptainer"
VM_OUT_DIR="${HOME}/apptainer-build"
COPY_BACK_DIR=""
JOBS=1
SUMMARY_FILE=""
BUILD_ONE_DEF=""
UBUNTU_BASE_SIF="${APPTAINER_UBUNTU_BASE_SIF:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --defs-dir) DEFS_DIR="$2"; shift 2 ;;
    --vm-out) VM_OUT_DIR="$2"; shift 2 ;;
    --copy-back) COPY_BACK_DIR="$2"; shift 2 ;;
    --jobs) JOBS="$2"; shift 2 ;;
    --summary-file) SUMMARY_FILE="$2"; shift 2 ;;
    --build-one) BUILD_ONE_DEF="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "required command not found: $cmd" >&2
    exit 127
  fi
}

require_cmd apptainer
require_cmd find
require_cmd sort
require_cmd mktemp
require_cmd sed

if [[ ! -d "$DEFS_DIR" ]]; then
  echo "defs dir not found: $DEFS_DIR" >&2
  exit 2
fi

mkdir -p "$VM_OUT_DIR/logs" "$VM_OUT_DIR/sif"
if [[ ! -w "$VM_OUT_DIR" ]]; then
  echo "vm output dir not writable: $VM_OUT_DIR" >&2
  exit 2
fi

if ! [[ "$JOBS" =~ ^[0-9]+$ ]] || [[ "$JOBS" -lt 1 ]]; then
  echo "jobs must be a positive integer: $JOBS" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VM_OUT_ABS="$(cd "$VM_OUT_DIR" && pwd)"
if [[ "$VM_OUT_ABS" == "$WORKSPACE_ROOT"* ]]; then
  echo "vm output dir must be outside workspace: $VM_OUT_ABS" >&2
  exit 2
fi

# Defensive guard: avoid building directly into workspace-mounted paths that can be read-only in VM setups.
case "$(cd "$VM_OUT_DIR" && pwd)" in
  /Volumes/*|/mnt/*)
    echo "vm output dir appears to be a host mount; choose VM-local writable path: $VM_OUT_DIR" >&2
    exit 2
    ;;
esac

build_one() {
  local def_file="$1"
  local name
  name="$(basename "$def_file" .def)"
  local sif="$VM_OUT_DIR/sif/${name}.sif"
  local log="$VM_OUT_DIR/logs/${name}.log"
  local tmp_def=""

  echo "[build] $name"
  rm -f "$sif" "$log"
  # Some Apptainer versions reject Docker refs with both tag and digest
  # (image:tag@sha256:...). Normalize to image@sha256 in a temp copy.
  tmp_def="$(mktemp "${TMPDIR:-/tmp}/apptainer-def-${name}.XXXXXX.def")"
  sed -E 's#^([[:space:]]*From:[[:space:]]*.+):([^:@[:space:]]+)@(sha256:[a-f0-9]+)[[:space:]]*$#\1@\3#' "$def_file" >"$tmp_def"
  if [[ -n "$UBUNTU_BASE_SIF" && -f "$UBUNTU_BASE_SIF" ]]; then
    if grep -Eq '^Bootstrap:[[:space:]]*docker[[:space:]]*$' "$tmp_def" && \
       grep -Eq '^From:[[:space:]]*(ubuntu(:[[:alnum:]._-]+)?@sha256:[a-f0-9]+|docker\.io/library/ubuntu(:[[:alnum:]._-]+)?@sha256:[a-f0-9]+)[[:space:]]*$' "$tmp_def"; then
      sed -Ei \
        -e 's#^Bootstrap:[[:space:]]*docker[[:space:]]*$#Bootstrap: localimage#' \
        -e "s#^From:[[:space:]].*\$#From: ${UBUNTU_BASE_SIF}#" \
        "$tmp_def"
    fi
  fi
  if apptainer build "$sif" "$tmp_def" >"$log" 2>&1; then
    rm -f "$tmp_def"
    echo "[ok] $name -> $sif"
  else
    rm -f "$tmp_def"
    echo "[fail] $name (see $log)" >&2
    return 1
  fi
}

export VM_OUT_DIR
export -f build_one

if [[ -n "$BUILD_ONE_DEF" ]]; then
  build_one "$BUILD_ONE_DEF"
  exit $?
fi

mapfile -t defs < <(find "$DEFS_DIR" -maxdepth 1 -type f -name '*.def' | sort)
if [[ "${#defs[@]}" -eq 0 ]]; then
  echo "no .def files found in $DEFS_DIR" >&2
  exit 2
fi

status=0
if [[ "$JOBS" -le 1 ]]; then
  for d in "${defs[@]}"; do
    if ! build_one "$d"; then
      status=1
    fi
  done
else
  require_cmd xargs
  if ! printf '%s\0' "${defs[@]}" | xargs -0 -P "$JOBS" -I {} \
    "$0" --build-one "{}" --defs-dir "$DEFS_DIR" --vm-out "$VM_OUT_DIR"; then
    status=1
  fi
fi

if [[ -n "$COPY_BACK_DIR" ]]; then
  mkdir -p "$COPY_BACK_DIR/sif" "$COPY_BACK_DIR/logs"
  cp -f "$VM_OUT_DIR"/sif/*.sif "$COPY_BACK_DIR/sif/" 2>/dev/null || true
  cp -f "$VM_OUT_DIR"/logs/*.log "$COPY_BACK_DIR/logs/" 2>/dev/null || true
  echo "copied outputs to $COPY_BACK_DIR"
fi

summary_path="${SUMMARY_FILE:-$VM_OUT_DIR/summary.tsv}"
{
  printf "tool\tstatus\tlog\n"
  for d in "${defs[@]}"; do
    name="$(basename "$d" .def)"
    log="$VM_OUT_DIR/logs/${name}.log"
    sif="$VM_OUT_DIR/sif/${name}.sif"
    if [[ -f "$sif" ]]; then
      printf "%s\tOK\t%s\n" "$name" "$log"
    else
      printf "%s\tFAIL\t%s\n" "$name" "$log"
    fi
  done
} >"$summary_path"

echo "build summary:"
column -t -s $'\t' "$summary_path" || cat "$summary_path"

echo "done: sif=$VM_OUT_DIR/sif logs=$VM_OUT_DIR/logs"
exit "$status"
