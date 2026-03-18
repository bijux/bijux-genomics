#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

./bin/require-isolate >/dev/null

base_ref="${LINT_FAST_BASE_REF:-}"
if [[ -z "$base_ref" ]]; then
  if git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    base_ref="HEAD~1"
  else
    base_ref="HEAD"
  fi
fi

changed="$(git diff --name-only "$base_ref"..HEAD || true)"
if [[ -z "$changed" ]]; then
  echo "lint-fast: no changed files; running config+script lint baseline"
  ./scripts/run.sh checks check-config-schema
  ./scripts/run.sh checks check-script-interface
  exit 0
fi

need_fmt=0
need_clippy=0
need_docs=0
need_configs=0
need_scripts=0

while IFS= read -r file; do
  [[ -z "$file" ]] && continue
  case "$file" in
    *.rs|Cargo.toml|Cargo.lock|crates/*)
      need_fmt=1
      need_clippy=1
      ;;
  esac
  case "$file" in
    docs/*|README.md|*/README.md)
      need_docs=1
      ;;
  esac
  case "$file" in
    configs/*|assets/reference/*)
      need_configs=1
      ;;
  esac
  case "$file" in
    scripts/*|makes/*|Makefile)
      need_scripts=1
      ;;
  esac
done <<< "$changed"

if [[ "$need_fmt" -eq 1 ]]; then
  echo "lint-fast: running rustfmt"
  ./scripts/run.sh tooling ci-fmt
fi

if [[ "$need_clippy" -eq 1 ]]; then
  echo "lint-fast: running clippy for executor/runtime subset"
  ./scripts/run.sh tooling ci-clippy-executors
fi

if [[ "$need_docs" -eq 1 ]]; then
  echo "lint-fast: running docs checks"
  ./scripts/run.sh docs check-doc-links
  ./scripts/run.sh checks check-docs-build-contract
fi

if [[ "$need_configs" -eq 1 ]]; then
  echo "lint-fast: running config checks"
  ./scripts/run.sh checks check-config-schema
  ./scripts/run.sh checks check-config-layout
fi

if [[ "$need_scripts" -eq 1 ]]; then
  echo "lint-fast: running script interface checks"
  ./scripts/run.sh checks check-script-interface
  ./scripts/run.sh checks check-clippy-allowlist-growth
  ./scripts/run.sh checks check-rustflags-consistency
fi

echo "lint-fast: OK"
