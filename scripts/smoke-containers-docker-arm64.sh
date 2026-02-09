#!/usr/bin/env bash
set -euo pipefail

# Build + smoke all Docker arm64 Dockerfile.* definitions.
# Artifacts:
#   artifacts/container/logs/docker-arm64/*.log
#   artifacts/container/images/docker-arm64/*.tar
#   artifacts/container/images/docker-arm64/images.txt
#
# Optional env:
#   DOCKER_BIN=docker
#   DOCKER_DIR=containers/docker/arm64
#   JOBS=1
#   SAVE_TAR=1
#   VERSION_TIMEOUT=120
#   IMAGE_PREFIX=bijux-smoke

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

DOCKER_BIN="${DOCKER_BIN:-docker}"
DOCKER_DIR="${DOCKER_DIR:-$ROOT_DIR/containers/docker/arm64}"
JOBS="${JOBS:-1}"
SAVE_TAR="${SAVE_TAR:-1}"
VERSION_TIMEOUT="${VERSION_TIMEOUT:-120}"
IMAGE_PREFIX="${IMAGE_PREFIX:-bijux-smoke}"

ARTIFACT_DIR="$ROOT_DIR/artifacts/container"
LOG_DIR="$ARTIFACT_DIR/logs/docker-arm64"
IMG_DIR="$ARTIFACT_DIR/images/docker-arm64"
SUMMARY="$LOG_DIR/summary.txt"
IMAGES_TXT="$IMG_DIR/images.txt"

mkdir -p "$LOG_DIR" "$IMG_DIR"

command -v "$DOCKER_BIN" >/dev/null 2>&1 || {
  echo "ERROR: '$DOCKER_BIN' not found" >&2
  exit 127
}

if [[ ! -d "$DOCKER_DIR" ]]; then
  echo "ERROR: docker dir not found: $DOCKER_DIR" >&2
  exit 2
fi

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
  local dockerfile="$1"
  local tool
  tool="$(basename "$dockerfile" | sed 's/^Dockerfile\.//')"
  local image="$IMAGE_PREFIX/${tool}:arm64"
  local log="$LOG_DIR/${tool}.log"
  local cmd
  cmd="$(get_version_cmd "$tool")"

  {
    echo "=== [$tool] build start"
    echo "dockerfile: $dockerfile"
    echo "image: $image"
    "$DOCKER_BIN" build -f "$dockerfile" -t "$image" "$DOCKER_DIR"
    echo "=== [$tool] smoke: $cmd"
    run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm "$image" sh -lc "$cmd"
    if [[ "$SAVE_TAR" = "1" ]]; then
      echo "=== [$tool] save image tar"
      "$DOCKER_BIN" save "$image" -o "$IMG_DIR/${tool}.tar"
    fi
    echo "$image" >> "$IMAGES_TXT"
    echo "=== [$tool] OK"
  } >"$log" 2>&1 || {
    echo "FAIL $tool (see $log)"
    return 1
  }

  echo "OK $tool"
}

export ROOT_DIR DOCKER_BIN DOCKER_DIR LOG_DIR IMG_DIR SAVE_TAR VERSION_TIMEOUT IMAGE_PREFIX IMAGES_TXT
export -f get_version_cmd
export -f build_and_smoke_one

mapfile -t dockerfiles < <(find "$DOCKER_DIR" -maxdepth 1 -type f -name 'Dockerfile.*' | sort)
if [[ "${#dockerfiles[@]}" -eq 0 ]]; then
  echo "ERROR: no Dockerfile.* found in $DOCKER_DIR" >&2
  exit 2
fi

: >"$SUMMARY"
: >"$IMAGES_TXT"
echo "Docker arm64 smoke run" | tee -a "$SUMMARY"
echo "dockerfiles: ${#dockerfiles[@]}" | tee -a "$SUMMARY"
echo "logs: $LOG_DIR" | tee -a "$SUMMARY"
echo "images: $IMG_DIR" | tee -a "$SUMMARY"

status=0
if [[ "$JOBS" -le 1 ]]; then
  for f in "${dockerfiles[@]}"; do
    build_and_smoke_one "$f" || status=1
  done
else
  printf '%s\n' "${dockerfiles[@]}" | xargs -I{} -P "$JOBS" bash -lc 'build_and_smoke_one "$@"' _ {} || status=1
fi

ok_count="$(grep -h '^=== .* OK$' "$LOG_DIR"/*.log 2>/dev/null | wc -l | tr -d ' ')"
fail_count=0
for f in "${dockerfiles[@]}"; do
  t="$(basename "$f" | sed 's/^Dockerfile\.//')"
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
