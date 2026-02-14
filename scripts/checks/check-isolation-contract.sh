#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

if ! command -v rg >/dev/null 2>&1; then
  echo "isolation-contract: ripgrep (rg) is required but not found in PATH" >&2
  exit 127
fi

tag_a="contract-a-$$"
tag_b="contract-b-$$"
root_a="$(env -u ISO_ROOT -u ISO_RUN_ID -u CARGO_TARGET_DIR -u CARGO_HOME -u TMPDIR -u TMP -u TEMP ISO_TAG="$tag_a" "$ROOT_DIR/bin/isolate" --print-root)"
root_b="$(env -u ISO_ROOT -u ISO_RUN_ID -u CARGO_TARGET_DIR -u CARGO_HOME -u TMPDIR -u TMP -u TEMP ISO_TAG="$tag_b" "$ROOT_DIR/bin/isolate" --print-root)"
tag_printed="$(env -u ISO_ROOT -u ISO_RUN_ID -u CARGO_TARGET_DIR -u CARGO_HOME -u TMPDIR -u TMP -u TEMP ISO_TAG="$tag_a" "$ROOT_DIR/bin/isolate" --print-tag)"

[[ "$tag_printed" == "$tag_a" ]] || {
  echo "isolation-contract: --print-tag returned unexpected value: $tag_printed" >&2
  exit 1
}

[[ "$root_a" != "$root_b" ]] || {
  echo "isolation-contract: two isolate tags produced the same ISO_ROOT" >&2
  exit 1
}

case "$root_a" in
  "$ROOT_DIR"/artifacts/isolates/*) ;;
  *)
    echo "isolation-contract: ISO_ROOT must be inside artifacts/isolates: $root_a" >&2
    exit 1
    ;;
esac
case "$root_b" in
  "$ROOT_DIR"/artifacts/isolates/*) ;;
  *)
    echo "isolation-contract: ISO_ROOT must be inside artifacts/isolates: $root_b" >&2
    exit 1
    ;;
esac

env_line="$(env -u ISO_ROOT -u ISO_RUN_ID -u CARGO_TARGET_DIR -u CARGO_HOME -u TMPDIR -u TMP -u TEMP ISO_TAG="$tag_a" "$ROOT_DIR/bin/isolate" sh -ceu 'printf "%s|%s|%s|%s|%s|%s|%s\n" "$ISO_TAG" "$ISO_ROOT" "$CARGO_TARGET_DIR" "$CARGO_HOME" "$TMPDIR" "$TMP" "$TEMP"')"
IFS='|' read -r env_tag env_root env_target env_home env_tmpdir env_tmp env_temp <<< "$env_line"

[[ -n "$env_tag" && -n "$env_root" && -n "$env_target" && -n "$env_home" && -n "$env_tmpdir" && -n "$env_tmp" && -n "$env_temp" ]] || {
  echo "isolation-contract: required isolate env vars are missing in command environment" >&2
  exit 1
}
[[ "$env_tag" == "$tag_a" ]] || {
  echo "isolation-contract: ISO_TAG mismatch in command environment" >&2
  exit 1
}
for path in "$env_target" "$env_home" "$env_tmpdir" "$env_tmp" "$env_temp"; do
  case "$path" in
    "$env_root"/*) ;;
    *)
      echo "isolation-contract: env path is outside ISO_ROOT: $path" >&2
      exit 1
      ;;
  esac
done

require_tag="contract-require-empty-$$"
env -u ISO_ROOT -u ISO_RUN_ID -u CARGO_TARGET_DIR -u CARGO_HOME -u TMPDIR -u TMP -u TEMP ISO_TAG="$require_tag" "$ROOT_DIR/bin/isolate" sh -ceu 'mkdir -p "$ISO_ROOT/target-test"'
if env -u ISO_ROOT -u ISO_RUN_ID -u CARGO_TARGET_DIR -u CARGO_HOME -u TMPDIR -u TMP -u TEMP ISO_TAG="$require_tag" "$ROOT_DIR/bin/isolate" --require-empty-target-dir sh -ceu 'true' >/dev/null 2>&1; then
  echo "isolation-contract: --require-empty-target-dir should fail when target-* exists and --reuse is not passed" >&2
  exit 1
fi
env -u ISO_ROOT -u ISO_RUN_ID -u CARGO_TARGET_DIR -u CARGO_HOME -u TMPDIR -u TMP -u TEMP ISO_TAG="$require_tag" "$ROOT_DIR/bin/isolate" --require-empty-target-dir --reuse sh -ceu 'true' >/dev/null

if rg -n "/Users/|[A-Za-z]:\\\\Users\\\\" crates/*/tests/snapshots >/dev/null 2>&1; then
  echo "absolute host paths leaked into snapshots" >&2
  rg -n "/Users/|[A-Za-z]:\\\\Users\\\\" crates/*/tests/snapshots >&2 || true
  exit 1
fi

echo "isolation-contract: OK"
