#!/usr/bin/env bash
set -euo pipefail

repo_root="${PROJECT_ROOT:-$(git rev-parse --show-toplevel)}"
shared_expression_builder="${repo_root}/.bijux/shared/bijux-makes-rs/scripts/nextest_expr.sh"

if [[ ! -x "${shared_expression_builder}" ]]; then
  echo "shared Nextest expression builder is unavailable: ${shared_expression_builder}" >&2
  exit 1
fi

# Shared roster entries are function names. Match them as complete integration-test
# names or as the final component of module-qualified unit-test names.
"${shared_expression_builder}" "$@" |
  sed 's#test(/\^(?:#test(/(?:^|::)(?:#g'
