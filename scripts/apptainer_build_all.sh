#!/usr/bin/env bash
set -euo pipefail

# Batch-build Apptainer defs in a VM-local writable directory.
# Sequential by default; optional limited concurrency.

DEFS_DIR="containers/apptainer"
VM_OUT_DIR="${HOME}/apptainer-build"
COPY_BACK_DIR=""
JOBS=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --defs-dir) DEFS_DIR="$2"; shift 2 ;;
    --vm-out) VM_OUT_DIR="$2"; shift 2 ;;
    --copy-back) COPY_BACK_DIR="$2"; shift 2 ;;
    --jobs) JOBS="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ ! -d "$DEFS_DIR" ]]; then
  echo "defs dir not found: $DEFS_DIR" >&2
  exit 2
fi

mkdir -p "$VM_OUT_DIR/logs" "$VM_OUT_DIR/sif"
if [[ ! -w "$VM_OUT_DIR" ]]; then
  echo "vm output dir not writable: $VM_OUT_DIR" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
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

  echo "[build] $name"
  if apptainer build "$sif" "$def_file" >"$log" 2>&1; then
    echo "[ok] $name -> $sif"
  else
    echo "[fail] $name (see $log)" >&2
    return 1
  fi
}

export VM_OUT_DIR
export -f build_one

mapfile -t defs < <(find "$DEFS_DIR" -maxdepth 1 -type f -name '*.def' | sort)
if [[ "${#defs[@]}" -eq 0 ]]; then
  echo "no .def files found in $DEFS_DIR" >&2
  exit 2
fi

if [[ "$JOBS" -le 1 ]]; then
  status=0
  for d in "${defs[@]}"; do
    if ! build_one "$d"; then
      status=1
    fi
  done
  [[ "$status" -eq 0 ]] || exit "$status"
else
  printf '%s\n' "${defs[@]}" | xargs -I{} -P "$JOBS" bash -lc 'build_one "$@"' _ {}
fi

if [[ -n "$COPY_BACK_DIR" ]]; then
  mkdir -p "$COPY_BACK_DIR/sif" "$COPY_BACK_DIR/logs"
  cp -f "$VM_OUT_DIR"/sif/*.sif "$COPY_BACK_DIR/sif/" 2>/dev/null || true
  cp -f "$VM_OUT_DIR"/logs/*.log "$COPY_BACK_DIR/logs/" 2>/dev/null || true
  echo "copied outputs to $COPY_BACK_DIR"
fi

echo "done: sif=$VM_OUT_DIR/sif logs=$VM_OUT_DIR/logs"
