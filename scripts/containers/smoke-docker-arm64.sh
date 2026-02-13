#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
source "$ROOT_DIR/scripts/_lib/common.sh"

DOCKER_BIN="${DOCKER_BIN:-docker}"
DOCKER_DIR="${DOCKER_DIR:-$ROOT_DIR/containers/docker/arm64}"
DOCKER_PLATFORM="${DOCKER_PLATFORM:-linux/arm64}"
DOCKER_ARCH="${DOCKER_ARCH:-arm64}"
RUNTIME_NAME="${RUNTIME_NAME:-docker-$DOCKER_ARCH}"
BIJUX_WORKERS="${BIJUX_WORKERS:-${JOBS:-1}}"
JOBS="${JOBS:-$BIJUX_WORKERS}"
SAVE_TAR="${SAVE_TAR:-1}"
VERSION_TIMEOUT="${VERSION_TIMEOUT:-120}"
IMAGE_PREFIX="${IMAGE_PREFIX:-bijux-smoke}"
TOOLS="${TOOLS:-}"
SMOKE_LEVEL="${SMOKE_LEVEL:-contract}"

ARTIFACT_DIR="${ARTIFACT_DIR:-$ROOT_DIR/artifacts/containers}"
LOG_DIR="$ARTIFACT_DIR/logs/$RUNTIME_NAME"
IMG_DIR="$ARTIFACT_DIR/images/$RUNTIME_NAME"
SUMMARY="$LOG_DIR/summary.txt"
IMAGES_TXT="$IMG_DIR/images.txt"
MANIFEST_DIR="$ARTIFACT_DIR"
INTERRUPTED=0

mkdir -p "$LOG_DIR" "$IMG_DIR" "$MANIFEST_DIR"

require_cmd "$DOCKER_BIN"
require_cmd jq
require_cmd python3
require_cmd awk
require_cmd sed
TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"

if ! "$DOCKER_BIN" info >/dev/null 2>&1; then
  echo "ERROR: cannot connect to docker daemon via $DOCKER_BIN" >&2
  echo "hint: start Docker Desktop/daemon and retry" >&2
  exit 2
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

cleanup_files() {
  rm -f "$LIST_FILE" "${TOOLS_FILE:-}" "${FILTERED_FILE:-}"
}

handle_interrupt() {
  INTERRUPTED=1
  echo "Interrupted; stopping container smoke run" >&2
  cleanup_files
  exit 130
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
  value=$("$ROOT_DIR/scripts/containers/registry-tools.sh" show-tool "$tool" 2>/dev/null \
    | jq -r --arg f "$field" '.[$f] // "unknown"' \
    | head -n 1 || true)
  if [ -n "${value:-}" ] && [ "$value" != "unknown" ]; then
    printf '%s\n' "$value"
    return 0
  fi

  # Fallback for VCF-only tools that are tracked in configs/ci/registry/tool_registry_vcf.toml.
  value=$(python3 - "$ROOT_DIR/configs/ci/registry/tool_registry_vcf.toml" "$tool" "$field" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
tool = sys.argv[2]
field = sys.argv[3]
if not path.exists():
    print("unknown")
    raise SystemExit(0)
try:
    import tomllib
except ModuleNotFoundError:
    print("unknown")
    raise SystemExit(0)
data = tomllib.loads(path.read_text())
for row in data.get("tools", []):
    if row.get("id") == tool:
        value = row.get(field, "unknown")
        if isinstance(value, (list, dict)) or value is None:
            print("unknown")
        else:
            print(str(value))
        raise SystemExit(0)
print("unknown")
PY
)
  printf '%s\n' "${value:-unknown}"
}

get_healthcheck_cmd() {
  tool="$1"
  value=$(get_registry_field healthcheck_cmd "$tool")
  if [ "$value" = "unknown" ] || [ -z "$value" ]; then
    get_help_cmd "$tool"
    return 0
  fi
  printf '%s\n' "$value"
}

get_expected_version_regex() {
  tool="$1"
  value=$(get_registry_field expected_version_regex "$tool")
  if [ "$value" = "unknown" ] || [ -z "$value" ]; then
    printf '%s\n' 'v?[0-9]+\.[0-9]+([.-][0-9A-Za-z]+)?'
    return 0
  fi
  printf '%s\n' "$value"
}

build_and_smoke_one() {
  dockerfile="$1"
  tool=$(basename "$dockerfile" | sed 's/^Dockerfile\.//')
  image="$IMAGE_PREFIX/${tool}:$DOCKER_ARCH"
  log="$LOG_DIR/${tool}.log"
  cmd=$(get_version_cmd "$tool")
  help_cmd=$(get_help_cmd "$tool")
  health_cmd=$(get_healthcheck_cmd "$tool")
  version_regex=$(get_expected_version_regex "$tool")
  expected_bin=$(get_registry_field expected_bin "$tool")
  if [ "$expected_bin" = "unknown" ] || [ -z "$expected_bin" ]; then
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
    if ! "$DOCKER_BIN" build --platform "$DOCKER_PLATFORM" \
      --build-arg "OCI_REVISION=$(git rev-parse HEAD 2>/dev/null || echo unknown)" \
      --build-arg "OCI_CREATED=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
      --build-arg "TOOL_VERSION=$declared_version" \
      -f "$dockerfile" -t "$image" "$DOCKER_DIR"; then
      echo "build failed for $tool"
      exit 1
    fi
    echo "=== [$tool] smoke-bin: $expected_bin"
    if ! run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm --entrypoint sh "$image" -lc "command -v $expected_bin >/dev/null"; then
      echo "binary missing in image: $expected_bin"
      exit 1
    fi
    echo "=== [$tool] smoke: $cmd"
    if ! run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm --entrypoint sh "$image" -lc "$cmd" >"$version_output_file" 2>&1; then
      cat "$version_output_file"
      echo "version command failed: $cmd"
      exit 1
    fi
    cat "$version_output_file"
    if [ ! -s "$version_output_file" ]; then
      echo "version command produced empty output: $cmd"
      exit 1
    fi
    if ! grep -Eiq "$version_regex" "$version_output_file"; then
      cat "$version_output_file"
      echo "version output does not match expected regex: $version_regex"
      exit 1
    fi
    if [ "$SMOKE_LEVEL" = "contract" ]; then
      echo "=== [$tool] smoke-help: $help_cmd"
      if ! run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm --entrypoint sh "$image" -lc "$help_cmd" >"$help_output_file" 2>&1; then
        cat "$help_output_file"
        echo "help command failed: $help_cmd"
        exit 1
      fi
      cat "$help_output_file"
      if [ ! -s "$help_output_file" ]; then
        echo "help command produced empty output: $help_cmd"
        exit 1
      fi
      echo "=== [$tool] healthcheck: $health_cmd"
      if ! run_with_timeout "$VERSION_TIMEOUT" "$DOCKER_BIN" run --rm --entrypoint sh "$image" -lc "$health_cmd" >/dev/null 2>&1; then
        echo "healthcheck failed: $health_cmd"
        exit 1
      fi
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

LIST_FILE=$(mktemp "$TMP_ROOT/dockerfiles.XXXXXX")
trap cleanup_files EXIT
trap handle_interrupt INT TERM
RUNTIME_TOOLS=$("$ROOT_DIR/scripts/containers/registry-tools.sh" tools-by-runtime docker)
if [ -z "${RUNTIME_TOOLS:-}" ]; then
  echo "ERROR: no docker runtime tools found in registry" >&2
  exit 2
fi
printf '%s\n' "$RUNTIME_TOOLS" \
  | tr ',' '\n' \
  | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' \
  | grep -v '^$' \
  | while IFS= read -r tool; do
      printf '%s/Dockerfile.%s\n' "$DOCKER_DIR" "$tool"
    done | sort > "$LIST_FILE"

if [ -n "$TOOLS" ]; then
  TOOLS_FILE=$(mktemp "$TMP_ROOT/docker-tools.XXXXXX")
  FILTERED_FILE=$(mktemp "$TMP_ROOT/dockerfiles-filtered.XXXXXX")
  printf '%s\n' "$TOOLS" | tr ',' '\n' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$' > "$TOOLS_FILE"
  MISSING_FILE=$(mktemp "$TMP_ROOT/docker-tools-missing.XXXXXX")
  awk -F/ '
    NR==FNR { wanted[$0]=1; next }
    {
      file=$NF
      sub(/^Dockerfile\./, "", file)
      found[file]=1
    }
    END {
      for (tool in wanted) {
        if (!(tool in found)) print tool
      }
    }
  ' "$TOOLS_FILE" "$LIST_FILE" | sort > "$MISSING_FILE"
  if [ -s "$MISSING_FILE" ]; then
    echo "ERROR: requested tools missing dockerfiles in $DOCKER_DIR:" >&2
    cat "$MISSING_FILE" >&2
    rm -f "$MISSING_FILE"
    exit 2
  fi
  rm -f "$MISSING_FILE"
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

MISSING_FILE=$(mktemp "$TMP_ROOT/docker-registry-missing.XXXXXX")
awk '
  {
    if (system("[ -f \"" $0 "\" ]") != 0) print $0
  }
' "$LIST_FILE" > "$MISSING_FILE"
if [ -s "$MISSING_FILE" ]; then
  echo "ERROR: registry tool dockerfiles missing under $DOCKER_DIR:" >&2
  cat "$MISSING_FILE" >&2
  rm -f "$MISSING_FILE"
  exit 2
fi
rm -f "$MISSING_FILE"

if [ ! -s "$LIST_FILE" ]; then
  echo "ERROR: no registry-driven dockerfile list for $DOCKER_DIR" >&2
  exit 2
fi

: >"$SUMMARY"
: >"$IMAGES_TXT"
echo "Docker $DOCKER_ARCH smoke run ($DOCKER_PLATFORM)" | tee -a "$SUMMARY"
echo "smoke_level: $SMOKE_LEVEL" | tee -a "$SUMMARY"
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

ok_count=0
fail_count=0
total_count=0
if [ ! -f "$LIST_FILE" ]; then
  echo "ERROR: dockerfile list missing before summary accounting" >&2
  status=1
else
while IFS= read -r f; do
  [ -n "$f" ] || continue
  t=$(basename "$f" | sed 's/^Dockerfile\.//')
  total_count=$((total_count + 1))
  if ! grep -q "=== \[$t\] OK" "$LOG_DIR/$t.log" 2>/dev/null; then
    fail_count=$((fail_count + 1))
  else
    ok_count=$((ok_count + 1))
  fi
done < "$LIST_FILE"
fi

echo "total: $total_count" | tee -a "$SUMMARY"
echo "ok: $ok_count" | tee -a "$SUMMARY"
echo "fail: $fail_count" | tee -a "$SUMMARY"

if [ "$INTERRUPTED" -ne 0 ] || [ "$fail_count" -ne 0 ] || [ "$status" -ne 0 ]; then
  echo "DONE with failures. inspect: $LOG_DIR" | tee -a "$SUMMARY"
  exit 1
fi

echo "DONE all passed" | tee -a "$SUMMARY"
