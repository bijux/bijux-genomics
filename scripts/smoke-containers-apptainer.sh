#!/bin/sh
set -eu
export TZ=UTC
export LC_ALL=C

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
SELF_SCRIPT="$SCRIPT_DIR/$(basename -- "$0")"
if [ ! -f "$SELF_SCRIPT" ]; then
  SELF_SCRIPT="$SCRIPT_DIR/smoke-containers-apptainer.sh"
fi

APPTAINER_BIN="${APPTAINER_BIN:-apptainer}"
DEFS_DIR="${DEFS_DIR:-$ROOT_DIR/containers/apptainer}"
VM_OUT_DIR="${VM_OUT_DIR:-$HOME/apptainer-smoke-build}"
JOBS="${JOBS:-1}"
BUILD_OPTS="${BUILD_OPTS:-}"
VERSION_TIMEOUT="${VERSION_TIMEOUT:-120}"
TOOLS="${TOOLS:-}"
SMOKE_LEVEL="${SMOKE_LEVEL:-version}"
SMOKE_RUN_MODE="${SMOKE_RUN_MODE:-bijux-run}"
UBUNTU_BASE_SIF="${APPTAINER_UBUNTU_BASE_SIF:-}"

ARTIFACT_DIR="${ARTIFACT_DIR:-$ROOT_DIR/artifacts/container}"
LOG_DIR="$ARTIFACT_DIR/logs/apptainer"
IMG_DIR="$ARTIFACT_DIR/images/apptainer"
SUMMARY="$LOG_DIR/summary.txt"
MANIFEST_DIR="$ARTIFACT_DIR"

mkdir -p "$LOG_DIR" "$IMG_DIR" "$VM_OUT_DIR/logs" "$VM_OUT_DIR/sif" "$MANIFEST_DIR"
export APPTAINER_BIN DEFS_DIR VM_OUT_DIR BUILD_OPTS VERSION_TIMEOUT TOOLS SMOKE_LEVEL
export SMOKE_RUN_MODE
export ARTIFACT_DIR LOG_DIR IMG_DIR SUMMARY MANIFEST_DIR ROOT_DIR SCRIPT_DIR
export UBUNTU_BASE_SIF REGISTRY_EXPORT_JSON

require_cmd() {
  name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "ERROR: required command '$name' not found in PATH" >&2
    exit 127
  fi
}

require_cmd "$APPTAINER_BIN"
require_cmd python3
require_cmd awk
require_cmd sed

if [ ! -d "$DEFS_DIR" ]; then
  echo "ERROR: defs dir not found: $DEFS_DIR" >&2
  exit 2
fi

case "$SMOKE_RUN_MODE" in
  bijux-run|apptainer-run) ;;
  *)
    echo "ERROR: unsupported SMOKE_RUN_MODE=$SMOKE_RUN_MODE (expected bijux-run|apptainer-run)" >&2
    exit 2
    ;;
esac

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
  if [ -z "${REGISTRY_EXPORT_JSON:-}" ]; then
    REGISTRY_EXPORT_JSON=$(./bin/isolate cargo run --bin bijux-dna -- registry export-json 2>/dev/null || true)
  fi
  if [ -z "${REGISTRY_EXPORT_JSON:-}" ]; then
    printf '%s\n' "unknown"
    return 0
  fi
  value=$(printf '%s\n' "$REGISTRY_EXPORT_JSON" | python3 - "$tool" "$field" <<'PY'
import json, sys
tool = sys.argv[1]
field = sys.argv[2]
try:
    payload = json.load(sys.stdin)
except Exception:
    print("unknown")
    raise SystemExit(0)
for item in payload.get("tools", []):
    if item.get("id") == tool:
        value = item.get(field, "unknown")
        if value is None:
            print("unknown")
        elif isinstance(value, (dict, list)):
            print("unknown")
        else:
            print(str(value))
        raise SystemExit(0)
print("unknown")
PY
)
  if [ -n "${value:-}" ] && [ "$value" != "unknown" ]; then
    printf '%s\n' "$value"
    return 0
  fi

  # Fallback for VCF-only tools that are tracked in configs/tool_registry_vcf.toml.
  value=$(python3 - "$ROOT_DIR/configs/tool_registry_vcf.toml" "$tool" "$field" <<'PY'
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

derive_runscript_args() {
  cmd="$1"
  expected_bin="$2"
  trimmed="$(printf '%s' "$cmd" | sed -E 's/^[[:space:]]+//;s/[[:space:]]+$//')"
  if [ -z "$trimmed" ]; then
    return 1
  fi
  # Strict parser: only allow "<expected_bin> <args...>" forms for apptainer run mode.
  first="$(printf '%s' "$trimmed" | awk '{print $1}')"
  if [ "$first" = "$expected_bin" ]; then
    printf '%s\n' "$(printf '%s' "$trimmed" | sed -E "s#^${expected_bin}([[:space:]]+|$)##")"
    return 0
  fi
  return 1
}

run_tool_command() {
  sif="$1"
  cmd="$2"
  expected_bin="$3"
  if [ "$SMOKE_RUN_MODE" = "apptainer-run" ]; then
    args="$(derive_runscript_args "$cmd" "$expected_bin" || true)"
    if [ -z "${args:-}" ] && [ "$cmd" != "$expected_bin" ]; then
      echo "cannot derive runscript args from command in apptainer-run mode: $cmd" >&2
      return 2
    fi
    if [ -n "${args:-}" ]; then
      # shellcheck disable=SC2086
      run_with_timeout "$VERSION_TIMEOUT" "$APPTAINER_BIN" run "$sif" $args
    else
      run_with_timeout "$VERSION_TIMEOUT" "$APPTAINER_BIN" run "$sif"
    fi
  else
    run_with_timeout "$VERSION_TIMEOUT" "$APPTAINER_BIN" exec "$sif" sh -lc "$cmd"
  fi
}

build_and_smoke_one() {
  def_file="$1"
  tmp_def=""
  tool=$(basename "$def_file" .def)
  vm_log="$VM_OUT_DIR/logs/${tool}.log"
  vm_sif="$VM_OUT_DIR/sif/${tool}.sif"
  out_log="$LOG_DIR/${tool}.log"
  out_sif="$IMG_DIR/${tool}.sif"
  cmd=$(get_version_cmd "$tool")
  help_cmd=$(get_help_cmd "$tool")
  health_cmd=$(get_healthcheck_cmd "$tool")
  version_regex=$(get_expected_version_regex "$tool")
  expected_bin=$(get_registry_field expected_bin "$tool")
  if [ "$expected_bin" = "unknown" ]; then
    expected_bin="$tool"
  fi
  version_output_file="$LOG_DIR/${tool}.version.out"
  help_output_file="$LOG_DIR/${tool}.help.out"
  manifest="$MANIFEST_DIR/${tool}.json"
  base_image=$(awk '/^From: /{print $2; exit}' "$def_file")
  upstream=$(get_registry_field upstream "$tool")
  pinned_commit=$(get_registry_field pinned_commit "$tool")
  declared_version=$(get_registry_field version "$tool")
  image_ref="$out_sif"
  image_digest="$(shasum -a 256 "$vm_sif" 2>/dev/null | awk '{print $1}' || true)"

  rm -f "$vm_sif" "$vm_log" "$out_log" "$out_sif" "$version_output_file" "$help_output_file"

  set +e
  {
    echo "=== [$tool] build start"
    echo "def: $def_file"
    echo "sif: $vm_sif"
    echo "mode: $SMOKE_RUN_MODE"
    tmp_def="$(mktemp "${TMPDIR:-/tmp}/apptainer-smoke-${tool}.XXXXXX.def")"
    sed -E 's#^([[:space:]]*From:[[:space:]]*.+):([^:@[:space:]]+)@(sha256:[a-f0-9]+)[[:space:]]*$#\1@\3#' "$def_file" > "$tmp_def"
    if [ -n "$UBUNTU_BASE_SIF" ] && [ -f "$UBUNTU_BASE_SIF" ]; then
      if grep -Eq '^Bootstrap:[[:space:]]*docker[[:space:]]*$' "$tmp_def" && \
         grep -Eq '^From:[[:space:]]*(ubuntu(:[[:alnum:]._-]+)?@sha256:[a-f0-9]+|docker\.io/library/ubuntu(:[[:alnum:]._-]+)?@sha256:[a-f0-9]+)[[:space:]]*$' "$tmp_def"; then
        sed -Ei \
          -e 's#^Bootstrap:[[:space:]]*docker[[:space:]]*$#Bootstrap: localimage#' \
          -e "s#^From:[[:space:]].*\$#From: ${UBUNTU_BASE_SIF}#" \
          "$tmp_def"
      fi
    fi
    # shellcheck disable=SC2086
    "$APPTAINER_BIN" build --force $BUILD_OPTS "$vm_sif" "$tmp_def"
    rm -f "$tmp_def"
    tmp_def=""
    echo "=== [$tool] smoke: $cmd"
    run_tool_command "$vm_sif" "$cmd" "$expected_bin" 2>&1 | tee "$version_output_file"
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
      run_tool_command "$vm_sif" "$help_cmd" "$expected_bin" 2>&1 | tee "$help_output_file"
      echo "=== [$tool] smoke-bin: $expected_bin"
      run_with_timeout "$VERSION_TIMEOUT" "$APPTAINER_BIN" exec "$vm_sif" sh -lc "command -v $expected_bin >/dev/null"
      echo "=== [$tool] healthcheck: $health_cmd"
      run_tool_command "$vm_sif" "$health_cmd" "$expected_bin" >/dev/null
    fi
    echo "=== [$tool] OK"
    version_output="$(head -n 1 "$version_output_file" 2>/dev/null | tr -d '\r')"
    version_output_json="$(json_escape "$version_output")"
    cmd_json="$(json_escape "$cmd")"
    def_json="$(json_escape "$def_file")"
    base_image_json="$(json_escape "$base_image")"
    image_json="$(json_escape "$out_sif")"
    declared_version_json="$(json_escape "$declared_version")"
    upstream_json="$(json_escape "$upstream")"
    pinned_commit_json="$(json_escape "$pinned_commit")"
    built_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    payload=$(cat <<JSON
{
  "tool": "$tool",
  "runtime": "apptainer",
  "status": "ok",
  "definition": "$def_json",
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
  } >"$vm_log" 2>&1
  run_status=$?
  if [ "$run_status" -ne 0 ]; then
    if [ -n "$tmp_def" ]; then
      rm -f "$tmp_def"
    fi
    cmd_json="$(json_escape "$cmd")"
    def_json="$(json_escape "$def_file")"
    base_image_json="$(json_escape "$base_image")"
    image_json="$(json_escape "$out_sif")"
    declared_version_json="$(json_escape "$declared_version")"
    upstream_json="$(json_escape "$upstream")"
    pinned_commit_json="$(json_escape "$pinned_commit")"
    built_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    payload=$(cat <<JSON
{
  "tool": "$tool",
  "runtime": "apptainer",
  "status": "fail",
  "definition": "$def_json",
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
    cp -f "$vm_log" "$out_log" 2>/dev/null || true
    echo "FAIL $tool (see $out_log)"
    set -e
    return 1
  fi
  set -e

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

if [ -n "$TOOLS" ]; then
  TOOLS_FILE=$(mktemp "${TMPDIR:-/tmp}/apptainer-tools.XXXXXX")
  FILTERED_FILE=$(mktemp "${TMPDIR:-/tmp}/apptainer-defs-filtered.XXXXXX")
  trap 'rm -f "$LIST_FILE" "$TOOLS_FILE" "$FILTERED_FILE"' EXIT INT TERM
  printf '%s\n' "$TOOLS" | tr ',' '\n' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$' > "$TOOLS_FILE"
  awk -F/ '
    NR==FNR { wanted[$0]=1; next }
    {
      file=$NF
      sub(/\.def$/, "", file)
      if (file in wanted) print $0
    }
  ' "$TOOLS_FILE" "$LIST_FILE" > "$FILTERED_FILE"
  mv "$FILTERED_FILE" "$LIST_FILE"
  rm -f "$TOOLS_FILE"
fi

if [ ! -s "$LIST_FILE" ]; then
  echo "ERROR: no .def files found in $DEFS_DIR" >&2
  exit 2
fi

: >"$SUMMARY"
echo "Apptainer smoke run" | tee -a "$SUMMARY"
echo "mode: $SMOKE_RUN_MODE" | tee -a "$SUMMARY"
echo "logs: $LOG_DIR" | tee -a "$SUMMARY"
echo "images: $IMG_DIR" | tee -a "$SUMMARY"

status=0
if [ "$JOBS" -le 1 ] 2>/dev/null; then
  while IFS= read -r d; do
    build_and_smoke_one "$d" || status=1
  done < "$LIST_FILE"
else
  xargs -P "$JOBS" -I{} sh "$SELF_SCRIPT" --worker {} < "$LIST_FILE" || status=1
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
