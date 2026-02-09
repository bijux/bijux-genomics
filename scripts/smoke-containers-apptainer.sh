#!/usr/bin/env bash
set -euo pipefail

# Build + smoke all Apptainer defs and collect artifacts.
# Artifacts:
#   artifacts/container/logs/apptainer/*.log
#   artifacts/container/images/apptainer/*.sif
#
# Optional env:
#   APPTAINER_BIN=apptainer
#   DEFS_DIR=containers/apptainer
#   VM_OUT_DIR=$HOME/apptainer-smoke-build   (must be writable, outside workspace)
#   JOBS=1
#   BUILD_OPTS="--fakeroot"
#   VERSION_TIMEOUT=120

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

APPTAINER_BIN="${APPTAINER_BIN:-apptainer}"
DEFS_DIR="${DEFS_DIR:-$ROOT_DIR/containers/apptainer}"
VM_OUT_DIR="${VM_OUT_DIR:-$HOME/apptainer-smoke-build}"
JOBS="${JOBS:-1}"
BUILD_OPTS="${BUILD_OPTS:-}"
VERSION_TIMEOUT="${VERSION_TIMEOUT:-120}"

ARTIFACT_DIR="$ROOT_DIR/artifacts/container"
LOG_DIR="$ARTIFACT_DIR/logs/apptainer"
IMG_DIR="$ARTIFACT_DIR/images/apptainer"
SUMMARY="$LOG_DIR/summary.txt"

mkdir -p "$LOG_DIR" "$IMG_DIR" "$VM_OUT_DIR/logs" "$VM_OUT_DIR/sif"

command -v "$APPTAINER_BIN" >/dev/null 2>&1 || {
  echo "ERROR: '$APPTAINER_BIN' not found" >&2
  exit 127
}

if [[ ! -d "$DEFS_DIR" ]]; then
  echo "ERROR: defs dir not found: $DEFS_DIR" >&2
  exit 2
fi

if [[ ! -w "$VM_OUT_DIR" ]]; then
  echo "ERROR: VM_OUT_DIR not writable: $VM_OUT_DIR" >&2
  exit 2
fi

VM_OUT_ABS="$(cd "$VM_OUT_DIR" && pwd)"
ROOT_ABS="$(cd "$ROOT_DIR" && pwd)"
if [[ "$VM_OUT_ABS" == "$ROOT_ABS"* ]]; then
  echo "ERROR: VM_OUT_DIR must be outside workspace: $VM_OUT_ABS" >&2
  exit 2
fi

case "$VM_OUT_ABS" in
  /Volumes/*|/mnt/*)
    echo "ERROR: VM_OUT_DIR appears host-mounted (likely read-only in VM): $VM_OUT_ABS" >&2
    exit 2
    ;;
esac

run_with_timeout() {
  local seconds="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$seconds" "$@"
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$seconds" "$@"
  else
    python3 - "$seconds" "$@" <<'PY'
import os, signal, subprocess, sys
timeout = int(sys.argv[1])
cmd = sys.argv[2:]
p = subprocess.Popen(cmd)
try:
    p.wait(timeout=timeout)
except subprocess.TimeoutExpired:
    p.send_signal(signal.SIGTERM)
    raise
sys.exit(p.returncode)
PY
  fi
}

get_version_cmd() {
  local tool="$1"
  python3 - "$ROOT_DIR/configs/tool_registry.toml" "$tool" <<'PY'
import sys, tomllib
path, tool = sys.argv[1], sys.argv[2]
with open(path, 'rb') as f:
    data = tomllib.load(f)
for entry in data.get('tools', []):
    if entry.get('id') == tool:
        print(entry.get('version_cmd', f"{tool} --version"))
        raise SystemExit(0)
print(f"{tool} --version")
PY
}

build_and_smoke_one() {
  local def_file="$1"
  local tool
  tool="$(basename "$def_file" .def)"
  local vm_log="$VM_OUT_DIR/logs/${tool}.log"
  local vm_sif="$VM_OUT_DIR/sif/${tool}.sif"
  local out_log="$LOG_DIR/${tool}.log"
  local out_sif="$IMG_DIR/${tool}.sif"
  local cmd
  cmd="$(get_version_cmd "$tool")"

  {
    echo "=== [$tool] build start"
    echo "def: $def_file"
    echo "sif: $vm_sif"
    "$APPTAINER_BIN" build --force $BUILD_OPTS "$vm_sif" "$def_file"
    echo "=== [$tool] smoke: $cmd"
    run_with_timeout "$VERSION_TIMEOUT" "$APPTAINER_BIN" exec "$vm_sif" sh -lc "$cmd"
    echo "=== [$tool] OK"
  } >"$vm_log" 2>&1 || {
    cp -f "$vm_log" "$out_log" 2>/dev/null || true
    echo "FAIL $tool (see $out_log)"
    return 1
  }

  cp -f "$vm_log" "$out_log"
  cp -f "$vm_sif" "$out_sif"
  echo "OK $tool"
}

export ROOT_DIR APPTAINER_BIN VM_OUT_DIR LOG_DIR IMG_DIR VERSION_TIMEOUT BUILD_OPTS
export -f get_version_cmd
export -f build_and_smoke_one

mapfile -t defs < <(find "$DEFS_DIR" -maxdepth 1 -type f -name '*.def' | sort)
if [[ "${#defs[@]}" -eq 0 ]]; then
  echo "ERROR: no .def files found in $DEFS_DIR" >&2
  exit 2
fi

: >"$SUMMARY"
echo "Apptainer smoke run" | tee -a "$SUMMARY"
echo "defs: ${#defs[@]}" | tee -a "$SUMMARY"
echo "logs: $LOG_DIR" | tee -a "$SUMMARY"
echo "images: $IMG_DIR" | tee -a "$SUMMARY"

status=0
if [[ "$JOBS" -le 1 ]]; then
  for d in "${defs[@]}"; do
    build_and_smoke_one "$d" || status=1
  done
else
  printf '%s\n' "${defs[@]}" | xargs -I{} -P "$JOBS" bash -lc 'build_and_smoke_one "$@"' _ {} || status=1
fi

ok_count="$(grep -h '^=== .* OK$' "$LOG_DIR"/*.log 2>/dev/null | wc -l | tr -d ' ')"
fail_count=0
for d in "${defs[@]}"; do
  t="$(basename "$d" .def)"
  if ! grep -q "=== \[$t\] OK" "$LOG_DIR/$t.log" 2>/dev/null; then
    fail_count=$((fail_count + 1))
  fi
done

echo "ok: $ok_count" | tee -a "$SUMMARY"
echo "fail: $fail_count" | tee -a "$SUMMARY"

if [[ "$fail_count" -ne 0 || "$status" -ne 0 ]]; then
  echo "DONE with failures. inspect: $LOG_DIR" | tee -a "$SUMMARY"
  exit 1
fi

echo "DONE all passed" | tee -a "$SUMMARY"
