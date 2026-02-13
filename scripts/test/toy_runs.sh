#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
export PYTHONPATH="$ROOT_DIR/scripts/tooling/python${PYTHONPATH:+:$PYTHONPATH}"
exec python3 -m bijux_dna_tools.toy_runs "$@"
