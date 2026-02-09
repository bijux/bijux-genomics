#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
DOCKER_ROOT="$ROOT_DIR/containers/docker"
APPTAINER_ROOT="$ROOT_DIR/containers/apptainer"

tmp=$(mktemp "${TMPDIR:-/tmp}/containers-lint.XXXXXX")
trap 'rm -f "$tmp"' EXIT INT TERM

record() {
  printf '%s\n' "$1" >> "$tmp"
}

check_header() {
  file="$1"
  if ! head -n 6 "$file" | grep -q 'SPDX-License-Identifier: GPL-3.0'; then
    record "$file: missing GPL SPDX header"
  fi
  if ! grep -q 'Container definition license: GPL-3.0' "$file"; then
    record "$file: missing container GPL license notice"
  fi
}

check_docker() {
  file="$1"
  base=$(basename "$file")
  case "$base" in
    Dockerfile.*) ;;
    *) record "$file: docker filename must match Dockerfile.<tool>" ;;
  esac

  check_header "$file"

  for key in \
    'org.opencontainers.image.source' \
    'org.opencontainers.image.revision' \
    'org.opencontainers.image.created' \
    'org.opencontainers.image.licenses' \
    'org.opencontainers.image.version'; do
    if ! grep -q "$key" "$file"; then
      record "$file: missing OCI label $key"
    fi
  done

  if ! grep -q 'org.opencontainers.image.licenses=.*GPL-3.0' "$file"; then
    record "$file: license label must declare GPL-3.0"
  fi

  if grep -qE 'FROM[[:space:]]+[^[:space:]]+:latest([[:space:]]|$)' "$file"; then
    record "$file: floating base tag latest is not allowed"
  fi
}

check_apptainer() {
  file="$1"
  base=$(basename "$file")
  case "$base" in
    *.def) ;;
    *) record "$file: apptainer filename must match <tool>.def" ;;
  esac

  check_header "$file"

  for key in \
    'org.opencontainers.image.source' \
    'org.opencontainers.image.revision' \
    'org.opencontainers.image.created' \
    'org.opencontainers.image.licenses' \
    'org.opencontainers.image.version'; do
    if ! grep -q "$key" "$file"; then
      record "$file: missing %labels key $key"
    fi
  done

  if grep -qiE 'apt(-get)?[[:space:]]+purge|apt(-get)?[[:space:]]+autoremove' "$file"; then
    record "$file: apptainer defs must not use purge/autoremove"
  fi

  if grep -qiE -- '-march=|-mavx|-mcpu=|-mtune=' "$file"; then
    if ! grep -q 'APPTAINER_CPU_FLAG_JUSTIFIED' "$file"; then
      record "$file: apptainer defs must not inject architecture flags"
    fi
  fi
}

find "$ROOT_DIR/containers" -type f -name '*.Dockerfile' | while IFS= read -r file; do
  record "$file: forbidden legacy docker naming (*.Dockerfile)"
done

find "$DOCKER_ROOT" -type f -name 'Dockerfile.*' | while IFS= read -r file; do
  check_docker "$file"
done

find "$APPTAINER_ROOT" -type f -name '*.def' | while IFS= read -r file; do
  check_apptainer "$file"
done

if [ -s "$tmp" ]; then
  echo "container lint violations:" >&2
  cat "$tmp" >&2
  exit 1
fi

echo "containers lint: ok"
