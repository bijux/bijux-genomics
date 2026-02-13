#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <fastq|bam>" >&2
  exit 2
fi

case "$1" in
  fastq)
    exec "$ROOT_DIR/scripts/smoke/smoke_fastq.sh"
    ;;
  bam)
    exec "$ROOT_DIR/scripts/smoke/smoke_bam.sh"
    ;;
  *)
    echo "unsupported smoke target: $1" >&2
    exit 2
    ;;
esac
