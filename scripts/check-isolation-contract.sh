#!/usr/bin/env bash
set -euo pipefail

if ! command -v rg >/dev/null 2>&1; then
  echo "isolation-contract: ripgrep (rg) is required but not found in PATH" >&2
  exit 127
fi

required=(TMPDIR TMP TEMP TEST_TARGET_DIR COV_TARGET_DIR TEST_TMP_DIR COV_TMP_DIR)
for var in "${required[@]}"; do
  if [[ -z "${!var:-}" ]]; then
    echo "missing required isolation env var: ${var}" >&2
    exit 1
  fi
done

case "${TEST_TARGET_DIR}" in
  artifacts/isolates/*) ;;
  *)
    echo "TEST_TARGET_DIR must be under artifacts/isolates/: ${TEST_TARGET_DIR}" >&2
    exit 1
    ;;
esac

for p in "${TMPDIR}" "${TMP}" "${TEMP}" "${TEST_TMP_DIR}" "${COV_TMP_DIR}"; do
  case "${p}" in
    *artifacts/isolates/*) ;;
    *)
      echo "tmp path is not isolated: ${p}" >&2
      exit 1
      ;;
  esac
done

if rg -n "/Users/|[A-Za-z]:\\\\Users\\\\" crates/*/tests/snapshots >/dev/null 2>&1; then
  echo "absolute host paths leaked into snapshots" >&2
  rg -n "/Users/|[A-Za-z]:\\\\Users\\\\" crates/*/tests/snapshots >&2 || true
  exit 1
fi

echo "isolation-contract: OK"
