#!/bin/sh
set -eu
export TZ=UTC
export LC_ALL=C

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

DOCKER_BIN="${DOCKER_BIN:-docker}"
DOCKER_DIR="${DOCKER_DIR:-$ROOT_DIR/containers/docker/arm64}"
DOCKER_PLATFORM="${DOCKER_PLATFORM:-linux/arm64}"
DOCKER_ARCH="${DOCKER_ARCH:-arm64}"
RUNTIME_NAME="${RUNTIME_NAME:-docker-$DOCKER_ARCH}"
JOBS="${JOBS:-1}"
SAVE_TAR="${SAVE_TAR:-1}"
VERSION_TIMEOUT="${VERSION_TIMEOUT:-120}"
IMAGE_PREFIX="${IMAGE_PREFIX:-bijux-smoke}"
TOOLS="${TOOLS:-}"
SMOKE_LEVEL="${SMOKE_LEVEL:-version}"

ARTIFACT_DIR="${ARTIFACT_DIR:-$ROOT_DIR/artifacts/container}"
LOG_DIR="$ARTIFACT_DIR/logs/$RUNTIME_NAME"
IMG_DIR="$ARTIFACT_DIR/images/$RUNTIME_NAME"
SUMMARY="$LOG_DIR/summary.txt"
IMAGES_TXT="$IMG_DIR/images.txt"
MANIFEST_DIR="$ARTIFACT_DIR"

mkdir -p "$LOG_DIR" "$IMG_DIR" "$MANIFEST_DIR"

require_cmd() {
  name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "ERROR: required command '$name' not found in PATH" >&2
    exit 127
  fi
}

require_cmd "$DOCKER_BIN"
require_cmd cargo
require_cmd awk
require_cmd sed

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
    require_cmd python3
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

write_manifest_json() {
  manifest_path="$1"
  payload="$2"
  tmp="${manifest_path}.tmp.$$"
  printf '%s\n' "$payload" > "$tmp"
  mv "$tmp" "$manifest_path"
}

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

get_version_cmd() {
  tool="$1"
  value=$(get_registry_field version_cmd "$tool")
  if [ "$value" = "unknown" ] || [ -z "$value" ]; then
    printf '%s\n' "$tool --version"
    return 0
  fi
  printf '%s\n' "$value"
}

get_help_cmd() {
  tool="$1"
  value=$(get_registry_field help_cmd "$tool")
  if [ "$value" = "unknown" ] || [ -z "$value" ]; then
    printf '%s\n' "$tool --help"
    return 0
  fi
  printf '%s\n' "$value"
}

get_registry_field() {
  field="$1"
  tool="$2"
  json=$(cargo run --bin bijux-dna -- registry show-tool "$tool" 2>/dev/null || true)
  if [ -z "$json" ]; then
    printf '%s\n' "unknown"
    return 0
  fi
  value=$(printf '%s\n' "$json" | awk -v field="$field" '
    BEGIN { found=0 }
    {
      key="\"" field "\""
      if (index($0, key) > 0) {
        line=$0
        sub(/^.*:[[:space:]]*/, "", line)
        sub(/[[:space:]]*,?[[:space:]]*$/, "", line)
        gsub(/^"/, "", line)
        gsub(/"$/, "", line)
        print line
        found=1
        exit 0
      }
    }
    END { if (!found) print "unknown" }
  ')
  printf '%s\n' "$value"
}

build_and_smoke_one() {
  dockerfile="$1"
  tool=$(basename "$dockerfile" | sed 's/^Dockerfile\.//')
  image="$IMAGE_PREFIX/${tool}:$DOCKER_ARCH"
  log="$LOG_DIR/${tool}.log"
  cmd=$(get_version_cmd "$tool")
  help_cmd=$(get_help_cmd "$tool")
  expected_bin=$(get_registry_field expected_bin "$tool")
  if [ "$expected_bin" = "unknown" ]; then
    expected_bin="$tool"
  fi
  version_output_file="$LOG_DIR/${tool}.version.out"
  help_output_file="$LOG_DIR/${tool}.help.out"
  manifest="$MANIFEST_DIR/${tool}.json"
  dockerfile_base=$(awk '/^FROM /{print $2; exit}' "$dockerfile")
  upstream=$(get_registry_field upstream "$tool")
  pinned_commit=$(get_registry_field pinned_commit "$tool")
  declared_version=$(get_registry_field version "$tool")
  image_ref="$image"
  image_digest="$("$DOCKER_BIN" image inspect --format '{{.Id}}' "$image" 2>/dev/null | head -n 1 || true)"

  {
    echo "=== [$tool] build start"
    echo "dockerfile: $dockerfile"
    echo "image: $image"
    "$DOCKER_BIN" build --platform "$DOCKER_PLATFORM" -f "$dockerfile" -t "$image" "$DOCKER_DIR"
    echo "=== [$tool] smoke: $cmd"
    run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm "$image" sh -lc "$cmd" | tee "$version_output_file"
    if [ "$SMOKE_LEVEL" = "contract" ]; then
      echo "=== [$tool] smoke-help: $help_cmd"
      run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm "$image" sh -lc "$help_cmd" | tee "$help_output_file"
      echo "=== [$tool] smoke-bin: $expected_bin"
      run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm "$image" sh -lc "command -v $expected_bin >/dev/null"
    fi
    if [ "$SAVE_TAR" = "1" ]; then
      echo "=== [$tool] save image tar"
      "$DOCKER_BIN" save "$image" -o "$IMG_DIR/${tool}.tar"
    fi
    echo "$image" >> "$IMAGES_TXT"
    echo "=== [$tool] OK"
    version_output="$(head -n 1 "$version_output_file" 2>/dev/null | tr -d '\r')"
    version_output_json="$(json_escape "$version_output")"
    cmd_json="$(json_escape "$cmd")"
    dockerfile_json="$(json_escape "$dockerfile")"
    base_image_json="$(json_escape "$dockerfile_base")"
    image_json="$(json_escape "$image")"
    declared_version_json="$(json_escape "$declared_version")"
    upstream_json="$(json_escape "$upstream")"
    pinned_commit_json="$(json_escape "$pinned_commit")"
    built_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    payload=$(cat <<JSON
{
  "tool": "$tool",
  "runtime": "$RUNTIME_NAME",
  "status": "ok",
  "dockerfile": "$dockerfile_json",
  "base_image": "$base_image_json",
  "image": "$image_json",
  "resolved_image_ref": "$(json_escape "$image_ref")",
  "resolved_image_digest": "$(json_escape "$image_digest")",
  "declared_version": "$declared_version_json",
  "upstream": "$upstream_json",
  "upstream_pin": "$pinned_commit_json",
  "version_command": "$cmd_json",
  "version_output": "$version_output_json",
  "built_at_utc": "$built_at"
}
JSON
)
    write_manifest_json "$manifest" "$payload"
  } >"$log" 2>&1 || {
    cmd_json="$(json_escape "$cmd")"
    dockerfile_json="$(json_escape "$dockerfile")"
    base_image_json="$(json_escape "$dockerfile_base")"
    image_json="$(json_escape "$image")"
    declared_version_json="$(json_escape "$declared_version")"
    upstream_json="$(json_escape "$upstream")"
    pinned_commit_json="$(json_escape "$pinned_commit")"
    built_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    payload=$(cat <<JSON
{
  "tool": "$tool",
  "runtime": "$RUNTIME_NAME",
  "status": "fail",
  "dockerfile": "$dockerfile_json",
  "base_image": "$base_image_json",
  "image": "$image_json",
  "resolved_image_ref": "$(json_escape "$image_ref")",
  "resolved_image_digest": "$(json_escape "$image_digest")",
  "declared_version": "$declared_version_json",
  "upstream": "$upstream_json",
  "upstream_pin": "$pinned_commit_json",
  "version_command": "$cmd_json",
  "version_output": "",
  "built_at_utc": "$built_at"
}
JSON
)
    write_manifest_json "$manifest" "$payload"
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

if [ -n "$TOOLS" ]; then
  TOOLS_FILE=$(mktemp "${TMPDIR:-/tmp}/docker-tools.XXXXXX")
  FILTERED_FILE=$(mktemp "${TMPDIR:-/tmp}/dockerfiles-filtered.XXXXXX")
  trap 'rm -f "$LIST_FILE" "$TOOLS_FILE" "$FILTERED_FILE"' EXIT INT TERM
  printf '%s\n' "$TOOLS" | tr ',' '\n' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$' > "$TOOLS_FILE"
  awk -F/ '
    NR==FNR { wanted[$0]=1; next }
    {
      file=$NF
      sub(/^Dockerfile\./, "", file)
      if (file in wanted) print $0
    }
  ' "$TOOLS_FILE" "$LIST_FILE" > "$FILTERED_FILE"
  mv "$FILTERED_FILE" "$LIST_FILE"
  rm -f "$TOOLS_FILE"
fi

if [ ! -s "$LIST_FILE" ]; then
  echo "ERROR: no Dockerfile.* found in $DOCKER_DIR" >&2
  exit 2
fi

: >"$SUMMARY"
: >"$IMAGES_TXT"
echo "Docker $DOCKER_ARCH smoke run ($DOCKER_PLATFORM)" | tee -a "$SUMMARY"
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
