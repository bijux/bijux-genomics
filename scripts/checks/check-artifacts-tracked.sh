#!/usr/bin/env bash
set -euo pipefail
LC_ALL=C
export LC_ALL

tracked="$(git ls-files artifacts || true)"
if [[ -n "${tracked}" ]]; then
  echo "tracked files under artifacts/ are forbidden:" >&2
  echo "${tracked}" >&2
  exit 1
fi

echo "artifacts-tracked-check: OK"
