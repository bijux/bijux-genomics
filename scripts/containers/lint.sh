#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
DOCKER_ROOT="$ROOT_DIR/containers/docker"
APPTAINER_ROOT="$ROOT_DIR/containers/apptainer"

TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
tmp=$(mktemp "$TMP_ROOT/tmp-containers-lint.XXXXXX")
trap 'rm -f "$tmp"' EXIT INT TERM

record() {
  printf '%s\n' "$1" >> "$tmp"
}

check_header() {
  file="$1"
  if ! head -n 6 "$file" | grep -q 'SPDX-License-Identifier:'; then
    record "$file: missing SPDX header"
  fi
  if ! grep -q 'Container definition license:' "$file"; then
    record "$file: missing container license notice"
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
    'org.opencontainers.image.title' \
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

  if ! grep -q 'org.opencontainers.image.licenses=' "$file"; then
    record "$file: license label must be present"
  fi

  if grep -qE 'FROM[[:space:]]+[^[:space:]]+:latest([[:space:]]|$)' "$file"; then
    record "$file: floating base tag latest is not allowed"
  fi
  from_image="$(awk '/^FROM /{print $2; exit}' "$file")"
  if ! printf '%s\n' "$from_image" | grep -q '@sha256:'; then
    record "$file: docker base image must be digest-pinned"
  fi
  base_repo="$(printf '%s\n' "$from_image" | sed -E 's/@sha256:.*$//' | sed -E 's/:.*$//')"
  case "$base_repo" in
    ubuntu|python|quay.io/biocontainers/bcftools) ;;
    *) record "$file: base image '$base_repo' is not allowed by containers/STYLE.md" ;;
  esac
}

check_apptainer() {
  file="$1"
  base=$(basename "$file")
  case "$base" in
    *.def) ;;
    *) record "$file: apptainer filename must match <tool>.def" ;;
  esac

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
  help_line=$(grep -n '^%help' "$file" | head -n1 | cut -d: -f1 || true)
  if [ -z "$labels_line" ] || [ -z "$env_line" ] || [ -z "$post_line" ] || [ -z "$run_line" ] || [ -z "$help_line" ]; then
    record "$file: required sections missing (need %labels, %environment, %post, %runscript, %help)"
  else
    if [ "$labels_line" -gt "$env_line" ] || [ "$env_line" -gt "$post_line" ] || [ "$post_line" -gt "$run_line" ] || [ "$run_line" -gt "$help_line" ]; then
      record "$file: section order must be %labels -> %environment -> %post -> %runscript -> %help"
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

"$SCRIPT_DIR/check-missing-images.sh"
"$SCRIPT_DIR/check-index.sh"
"$SCRIPT_DIR/check-non-bijux-sources.sh"
"$SCRIPT_DIR/check-owners.sh"
"$SCRIPT_DIR/check-promotion-policy.sh"
"$SCRIPT_DIR/check-version-completeness.sh"
"$SCRIPT_DIR/check-version-authority.sh"
"$SCRIPT_DIR/check-version-hash-pin.sh"
"$SCRIPT_DIR/check-version-lock.sh"
"$SCRIPT_DIR/check-lock-matches-built-output.sh"
"$SCRIPT_DIR/check-tool-id-manifest.sh"
"$SCRIPT_DIR/check-tool-id-contract.sh"
"$SCRIPT_DIR/check-registry-vs-defs.sh"
"$SCRIPT_DIR/check-tool-name-collision.sh"
"$SCRIPT_DIR/check-apptainer-bijux-header.sh"
"$SCRIPT_DIR/check-apptainer-hardening.sh"
"$SCRIPT_DIR/check-docker-labels.sh"
"$SCRIPT_DIR/check-docker-hardening.sh"
"$SCRIPT_DIR/check-docker-context.sh"
"$SCRIPT_DIR/check-smoke-contract.sh"
"$SCRIPT_DIR/check-build-provenance.sh"
"$SCRIPT_DIR/check-bijux-template-markers.sh"
"$SCRIPT_DIR/check-license-metadata.sh"
"$SCRIPT_DIR/check-docker-arch-policy.sh"
"$SCRIPT_DIR/check-digest-output-policy.sh"

echo "containers lint: ok"
