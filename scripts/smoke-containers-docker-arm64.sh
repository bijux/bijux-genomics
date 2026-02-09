#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

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

if ! command -v "$DOCKER_BIN" >/dev/null 2>&1; then
  echo "ERROR: '$DOCKER_BIN' not found" >&2
  exit 127
fi

if [ ! -d "$DOCKER_DIR" ]; then
  echo "ERROR: docker dir not found: $DOCKER_DIR" >&2
  exit 2
fi

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
  dockerfile="$1"
  tool=$(basename "$dockerfile" | sed 's/^Dockerfile\.//')
  image="$IMAGE_PREFIX/${tool}:arm64"
  log="$LOG_DIR/${tool}.log"
  cmd=$(get_version_cmd "$tool")

  {
    echo "=== [$tool] build start"
    echo "dockerfile: $dockerfile"
    echo "image: $image"
    "$DOCKER_BIN" build -f "$dockerfile" -t "$image" "$DOCKER_DIR"
    echo "=== [$tool] smoke: $cmd"
    run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm "$image" sh -lc "$cmd"
    if [ "$SAVE_TAR" = "1" ]; then
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

if [ "${1:-}" = "--worker" ]; then
  build_and_smoke_one "$2"
  exit $?
fi

LIST_FILE=$(mktemp "${TMPDIR:-/tmp}/dockerfiles.XXXXXX")
trap 'rm -f "$LIST_FILE"' EXIT INT TERM
find "$DOCKER_DIR" -maxdepth 1 -type f -name 'Dockerfile.*' | sort > "$LIST_FILE"

if [ ! -s "$LIST_FILE" ]; then
  echo "ERROR: no Dockerfile.* found in $DOCKER_DIR" >&2
  exit 2
fi

: >"$SUMMARY"
: >"$IMAGES_TXT"
echo "Docker arm64 smoke run" | tee -a "$SUMMARY"
echo "logs: $LOG_DIR" | tee -a "$SUMMARY"
echo "images: $IMG_DIR" | tee -a "$SUMMARY"

status=0
if [ "$JOBS" -le 1 ] 2>/dev/null; then
  while IFS= read -r f; do
    build_and_smoke_one "$f" || status=1
  done < "$LIST_FILE"
else
  xargs -P "$JOBS" -I{} sh "$0" --worker {} < "$LIST_FILE" || status=1
fi

ok_count=$(grep -h '^=== .* OK$' "$LOG_DIR"/*.log 2>/dev/null | wc -l | tr -d ' ')
fail_count=0
while IFS= read -r f; do
  t=$(basename "$f" | sed 's/^Dockerfile\.//')
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
