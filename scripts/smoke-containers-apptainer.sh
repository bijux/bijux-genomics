#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

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

if ! command -v "$APPTAINER_BIN" >/dev/null 2>&1; then
  echo "ERROR: '$APPTAINER_BIN' not found" >&2
  exit 127
fi

if [ ! -d "$DEFS_DIR" ]; then
  echo "ERROR: defs dir not found: $DEFS_DIR" >&2
  exit 2
fi

if [ ! -w "$VM_OUT_DIR" ]; then
  echo "ERROR: VM_OUT_DIR not writable: $VM_OUT_DIR" >&2
  exit 2
fi

VM_OUT_ABS=$(CDPATH= cd -- "$VM_OUT_DIR" && pwd)
ROOT_ABS=$(CDPATH= cd -- "$ROOT_DIR" && pwd)
case "$VM_OUT_ABS" in
  "$ROOT_ABS"/*)
    echo "ERROR: VM_OUT_DIR must be outside workspace: $VM_OUT_ABS" >&2
    exit 2
    ;;
  /Volumes/*|/mnt/*)
    echo "ERROR: VM_OUT_DIR appears host-mounted (likely read-only in VM): $VM_OUT_ABS" >&2
    exit 2
    ;;
esac

run_with_timeout() {
  seconds="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$seconds" "$@"
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$seconds" "$@"
  else
    python3 - "$seconds" "$@" <<'PY'
import signal, subprocess, sys
p = subprocess.Popen(sys.argv[2:])
try:
    p.wait(timeout=int(sys.argv[1]))
except subprocess.TimeoutExpired:
    p.send_signal(signal.SIGTERM)
    raise
sys.exit(p.returncode)
PY
  fi
}

get_version_cmd() {
  tool="$1"
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
  def_file="$1"
  tool=$(basename "$def_file" .def)
  vm_log="$VM_OUT_DIR/logs/${tool}.log"
  vm_sif="$VM_OUT_DIR/sif/${tool}.sif"
  out_log="$LOG_DIR/${tool}.log"
  out_sif="$IMG_DIR/${tool}.sif"
  cmd=$(get_version_cmd "$tool")

  {
    echo "=== [$tool] build start"
    echo "def: $def_file"
    echo "sif: $vm_sif"
    # shellcheck disable=SC2086
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

if [ "${1:-}" = "--worker" ]; then
  build_and_smoke_one "$2"
  exit $?
fi

LIST_FILE=$(mktemp "${TMPDIR:-/tmp}/apptainer-defs.XXXXXX")
trap 'rm -f "$LIST_FILE"' EXIT INT TERM
find "$DEFS_DIR" -maxdepth 1 -type f -name '*.def' | sort > "$LIST_FILE"

if [ ! -s "$LIST_FILE" ]; then
  echo "ERROR: no .def files found in $DEFS_DIR" >&2
  exit 2
fi

: >"$SUMMARY"
echo "Apptainer smoke run" | tee -a "$SUMMARY"
echo "logs: $LOG_DIR" | tee -a "$SUMMARY"
echo "images: $IMG_DIR" | tee -a "$SUMMARY"

status=0
if [ "$JOBS" -le 1 ] 2>/dev/null; then
  while IFS= read -r d; do
    build_and_smoke_one "$d" || status=1
  done < "$LIST_FILE"
else
  xargs -P "$JOBS" -I{} sh "$0" --worker {} < "$LIST_FILE" || status=1
fi

ok_count=$(grep -h '^=== .* OK$' "$LOG_DIR"/*.log 2>/dev/null | wc -l | tr -d ' ')
fail_count=0
while IFS= read -r d; do
  t=$(basename "$d" .def)
  if ! grep -q "=== \[$t\] OK" "$LOG_DIR/$t.log" 2>/dev/null; then
    fail_count=$((fail_count + 1))
  fi
done < "$LIST_FILE"

echo "ok: $ok_count" | tee -a "$SUMMARY"
echo "fail: $fail_count" | tee -a "$SUMMARY"

if [ "$fail_count" -ne 0 ] || [ "$status" -ne 0 ]; then
  echo "DONE with failures. inspect: $LOG_DIR" | tee -a "$SUMMARY"
  exit 1
fi

echo "DONE all passed" | tee -a "$SUMMARY"
