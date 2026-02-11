#!/usr/bin/env sh
set -eu

fail() {
  echo "ssot-guardrails: $*" >&2
  exit 1
}

changed_files="$(git show --name-only --pretty='' HEAD 2>/dev/null || true)"

if [ -n "$changed_files" ]; then
  if printf '%s\n' "$changed_files" | grep -qx 'configs/tool_registry.toml'; then
    if ! printf '%s\n' "$changed_files" | grep -qx 'configs/tool_registry.lock.sha256'; then
      fail "partial registry edit detected: configs/tool_registry.toml changed without configs/tool_registry.lock.sha256"
    fi
  fi

  if printf '%s\n' "$changed_files" | grep -Eq '^configs/stages.*\.toml$'; then
    if ! printf '%s\n' "$changed_files" | grep -Eq '^configs/param_registry.*\.toml$'; then
      fail "partial stage edit detected: stages*.toml changed without param_registry*.toml"
    fi
  fi
fi

echo "ssot-guardrails: OK"
