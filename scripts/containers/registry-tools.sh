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
cd "$ROOT_DIR"
source "$ROOT_DIR/scripts/_lib/common.sh"

./bin/require-isolate >/dev/null || {
  ./bin/require-isolate --explain >&2
  exit 1
}

cmd="${1:-}"
case "$cmd" in
  list-tools)
    cargo run --bin bijux-dna -- registry list-tools
    ;;
  list-stages)
    cargo run --bin bijux-dna -- registry list-stages
    ;;
  show-tool)
    tool_id="${2:-}"
    if [ -z "$tool_id" ]; then
      echo "usage: $0 show-tool <tool-id>" >&2
      exit 2
    fi
    cargo run --bin bijux-dna -- registry show-tool "$tool_id"
    ;;
  stage-tools)
    stage_id="${2:-}"
    kind="${3:-all}"
    if [ -z "$stage_id" ]; then
      echo "usage: $0 stage-tools <stage-id> [all|primary|optional|validation|reporting]" >&2
      exit 2
    fi
    cargo run --bin bijux-dna -- registry list-tools --stage "$stage_id" --kind "$kind"
    ;;
  tools-by-runtime)
    runtime="${2:-}"
    if [ -z "$runtime" ]; then
      echo "usage: $0 tools-by-runtime <docker|apptainer>" >&2
      exit 2
    fi
    case "$runtime" in
      docker|apptainer)
        ;;
      *)
        echo "unsupported runtime: $runtime" >&2
        exit 2
        ;;
    esac
    require_cmd jq
    cargo run --bin bijux-dna -- registry export-containers --json \
      | jq -r --arg runtime "$runtime" '
          .containers
          | map(select((.runtimes // []) | index($runtime)))
          | map(.tool_id)
          | unique
          | .[]
        ' \
      | paste -sd, -
    ;;
  *)
    echo "unknown command: $cmd" >&2
    exit 2
    ;;
esac
