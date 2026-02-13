#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

status=0

is_binaryish() {
  case "$1" in
    *.fastq|*.fq|*.bam|*.sam|*.vcf|*.bcf|*.fa|*.fasta|*.fa.gz|*.fasta.gz|*.vcf.gz|*.bam.bai|*.crai|*.tbi|*.sif|*.tar|*.tar.gz|*.zip|*.xz|*.bgz)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

while IFS= read -r toy_dir; do
  [[ -d "$toy_dir" ]] || continue
  checksum_file="$toy_dir/CHECKSUMS.sha256"
  rel="${checksum_file#"$ROOT_DIR/"}"
  if [[ ! -f "$checksum_file" ]]; then
    echo "asset-checksums: missing $rel" >&2
    status=1
    continue
  fi
  (
    cd "$toy_dir"
    shasum -a 256 -c CHECKSUMS.sha256 >/dev/null
  ) || {
    echo "asset-checksums: checksum mismatch in ${toy_dir#"$ROOT_DIR/"}" >&2
    status=1
  }
done < <(find "$ROOT_DIR/assets/toy" -mindepth 1 -maxdepth 1 -type d | sort)

while IFS= read -r golden_dir; do
  [[ -d "$golden_dir" ]] || continue
  required="$golden_dir/artifact_checksums.json"
  if [[ ! -f "$required" ]]; then
    echo "asset-checksums: missing ${required#"$ROOT_DIR/"}" >&2
    status=1
  fi
done < <(find "$ROOT_DIR/assets/golden/toy-runs-v1" -mindepth 1 -maxdepth 1 -type d | sort)

while IFS= read -r d; do
  [[ -d "$d" ]] || continue
  has_binary=0
  while IFS= read -r f; do
    [[ -f "$f" ]] || continue
    relf="${f#"$d/"}"
    case "$relf" in
      CHECKSUMS.sha256|GENERATE.md|README.md|index.md|*.md|*.toml|*.yaml|*.yml|*.json|*.txt)
        continue
        ;;
    esac
    if is_binaryish "$f"; then
      has_binary=1
      break
    fi
  done < <(find "$d" -maxdepth 1 -type f | sort)

  if [[ "$has_binary" -eq 1 && ! -f "$d/CHECKSUMS.sha256" ]]; then
    echo "asset-checksums: missing CHECKSUMS.sha256 in ${d#"$ROOT_DIR/"} for binary-ish assets" >&2
    status=1
  fi
done < <(find "$ROOT_DIR/assets" -type d | sort)

if [[ "$status" -ne 0 ]]; then
  exit "$status"
fi

echo "asset-checksums: OK"
