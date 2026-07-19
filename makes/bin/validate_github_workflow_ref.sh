#!/usr/bin/env bash
set -euo pipefail

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required tool is unavailable: $1" >&2
    exit 1
  fi
}

require_tool git
require_tool grep

repo_root="$(git rev-parse --show-toplevel)"
pinned_ref="${PINNED_REF:-${TEST_ALL_FROZEN_REF:-HEAD}}"

if ! full_sha="$(git -C "${repo_root}" rev-parse "${pinned_ref}^{commit}" 2>/dev/null)"; then
  echo "GitHub workflow ref does not resolve to a commit: ${pinned_ref}" >&2
  exit 2
fi

runner_path="makes/bin/run_github_workflow_gate.sh"
make_contract_path="makes/cargo.mk"

if ! git -C "${repo_root}" cat-file -e "${full_sha}:${runner_path}" 2>/dev/null; then
  echo "GitHub workflow ref does not provide ${runner_path}: ${pinned_ref}" >&2
  echo "choose a ref that implements the github-all gate" >&2
  exit 2
fi

if ! make_contract="$(git -C "${repo_root}" show "${full_sha}:${make_contract_path}" 2>/dev/null)"; then
  echo "GitHub workflow ref does not provide ${make_contract_path}: ${pinned_ref}" >&2
  echo "choose a ref that implements the github-all gate" >&2
  exit 2
fi

if ! grep -Eq '^github-all:[[:space:]]' <<<"${make_contract}"; then
  echo "GitHub workflow ref does not declare the github-all target: ${pinned_ref}" >&2
  echo "choose a ref that implements the github-all gate" >&2
  exit 2
fi
