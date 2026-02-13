#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tag_a="strong-a-$$"
tag_b="strong-b-$$"

root_a="$("$ROOT_DIR/bin/isolate" --tag "$tag_a" --print-root)"
root_b="$("$ROOT_DIR/bin/isolate" --tag "$tag_b" --print-root)"
[[ "$root_a" != "$root_b" ]] || {
  echo "isolate-strong: roots must differ for different tags" >&2
  exit 1
}

case "$root_a" in
  "$ROOT_DIR"/artifacts/isolates/*"$tag_a"*) ;;
  *)
    echo "isolate-strong: root_a does not include expected tag/path: $root_a" >&2
    exit 1
    ;;
esac
case "$root_b" in
  "$ROOT_DIR"/artifacts/isolates/*"$tag_b"*) ;;
  *)
    echo "isolate-strong: root_b does not include expected tag/path: $root_b" >&2
    exit 1
    ;;
esac

env_out="$("$ROOT_DIR/bin/isolate" --tag "$tag_a" --print-env)"
for k in ISO_TAG ISO_ROOT CARGO_TARGET_DIR CARGO_HOME TMPDIR TMP TEMP TZ LC_ALL RUST_BACKTRACE CARGO_TERM_COLOR; do
  grep -q "^${k}=" <<<"$env_out" || {
    echo "isolate-strong: --print-env missing ${k}" >&2
    exit 1
  }
done

"$ROOT_DIR/bin/isolate" --tag "$tag_a" --require-clean sh -ceu 'true'
if "$ROOT_DIR/bin/isolate" --tag "$tag_a" --require-clean sh -ceu 'true' >/dev/null 2>&1; then
  echo "isolate-strong: --require-clean should fail when ISO_ROOT already exists" >&2
  exit 1
fi
"$ROOT_DIR/bin/isolate" --tag "$tag_a" --require-clean --reuse sh -ceu 'true'

for d in target cargo-home tmp logs out; do
  [[ -d "$root_a/$d" ]] || {
    echo "isolate-strong: missing required isolate subdir $d under $root_a" >&2
    exit 1
  }
done

if "$ROOT_DIR/bin/isolate" --tag "$tag_b" bash -ceu '
  source "'"$ROOT_DIR"'/scripts/_lib/common.sh"
  ensure_artifacts_dir "'"$ROOT_DIR"'/tmp-outside-check" >/dev/null 2>&1
'; then
  echo "isolate-strong: ensure_artifacts_dir allowed path outside artifacts/ and ISO_ROOT" >&2
  exit 1
fi

echo "isolate-strong: OK"
