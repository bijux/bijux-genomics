#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
fail() {
  echo "ssot-guardrails: $*" >&2
  exit 1
}

changed_files="$(git show --name-only --pretty='' HEAD 2>/dev/null || true)"

if [ -n "$changed_files" ]; then
  if printf '%s\n' "$changed_files" | grep -qx 'configs/ci/registry/tool_registry.toml'; then
    if ! printf '%s\n' "$changed_files" | grep -qx 'configs/ci/registry/tool_registry_lock.sha256'; then
      fail "partial registry edit detected: configs/ci/registry/tool_registry.toml changed without configs/ci/registry/tool_registry_lock.sha256"
    fi
  fi

  if printf '%s\n' "$changed_files" | grep -Eq '^configs/ci/stages/.*\.toml$'; then
    if ! printf '%s\n' "$changed_files" | grep -Eq '^configs/ci/params/param_registry.*\.toml$'; then
      fail "partial stage edit detected: stages*.toml changed without param_registry*.toml"
    fi
  fi
fi

echo "ssot-guardrails: OK"
