#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
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
    'org.opencontainers.image.version' \
    'org.opencontainers.image.tool' \
    'org.opencontainers.image.base.name' \
    'org.opencontainers.image.base.digest'; do
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
  if ! awk '/^FROM /{print $2; exit}' "$file" | grep -q '@sha256:'; then
    record "$file: docker base image must be digest-pinned"
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
    'org.opencontainers.image.version' \
    'org.opencontainers.image.tool'; do
    if ! grep -q "$key" "$file"; then
      record "$file: missing %labels key $key"
    fi
  done

  if grep -qiE 'apt(-get)?[[:space:]]+purge|apt(-get)?[[:space:]]+autoremove|apt-mark[[:space:]]+auto' "$file"; then
    record "$file: apptainer defs must not use purge/autoremove/apt-mark auto"
  fi

  if grep -qiE -- '-march=|-mavx|-mcpu=|-mtune=' "$file"; then
    if ! grep -q 'APPTAINER_CPU_FLAG_JUSTIFIED' "$file"; then
      record "$file: apptainer defs must not inject architecture flags"
    fi
  fi

  labels_line=$(grep -n '^%labels' "$file" | head -n1 | cut -d: -f1 || true)
  env_line=$(grep -n '^%environment' "$file" | head -n1 | cut -d: -f1 || true)
  post_line=$(grep -n '^%post' "$file" | head -n1 | cut -d: -f1 || true)
  run_line=$(grep -n '^%runscript' "$file" | head -n1 | cut -d: -f1 || true)
  if [ -z "$labels_line" ] || [ -z "$env_line" ] || [ -z "$post_line" ] || [ -z "$run_line" ]; then
    record "$file: required sections missing (need %labels, %environment, %post, %runscript)"
  else
    if [ "$labels_line" -gt "$env_line" ] || [ "$env_line" -gt "$post_line" ] || [ "$post_line" -gt "$run_line" ]; then
      record "$file: section order must be %labels -> %environment -> %post -> %runscript"
    fi
  fi

  run_chunk=$(awk 'BEGIN{inside=0} /^%runscript/{inside=1; next} /^%[a-z]/{if(inside){exit}} {if(inside) print}' "$file")
  if ! printf '%s\n' "$run_chunk" | grep -q 'exec '; then
    if ! grep -q 'RUNSCRIPT_WRAPPER_JUSTIFIED' "$file"; then
      record "$file: %runscript must use exec"
    fi
  fi
  if ! printf '%s\n' "$run_chunk" | grep -Fq '"$@"'; then
    if ! grep -q 'RUNSCRIPT_WRAPPER_JUSTIFIED' "$file"; then
      record "$file: %runscript must exec tool with \"\$@\" passthrough"
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
