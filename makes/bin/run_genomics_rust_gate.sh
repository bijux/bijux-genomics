#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
shared_gate="${repo_root}/.bijux/shared/bijux-makes-rs/scripts/rust_gate.sh"

if [[ ! -x "${shared_gate}" ]]; then
  echo "shared Rust gate is unavailable: ${shared_gate}" >&2
  exit 1
fi

if [[ "${1:-}" == "audit" ]]; then
  allowlist="${repo_root}/audit-allowlist.toml"
  if [[ -f "${allowlist}" ]]; then
    audit_ignore_args="$(
      {
        grep -Eo 'RUSTSEC-[0-9]{4}-[0-9]{4}' "${allowlist}" || {
          grep_status=$?
          if [[ "${grep_status}" -ne 1 ]]; then
            exit "${grep_status}"
          fi
        }
      } |
        sort -u |
        sed 's/^/--ignore /' |
        paste -sd ' ' -
    )"
    export RUST_AUDIT_IGNORE_ARGS="${audit_ignore_args}"
  fi
fi

if [[ "${1:-}" == "test-all" ]]; then
  exec cargo run -q -p bijux-dna-dev -- tooling run ci-test-all-sharded
fi

exec "${shared_gate}" "$@"
