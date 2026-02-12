#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT_DIR"

if ! ./bin/require-isolate >/dev/null 2>&1; then
  exec ./bin/isolate "$0" "$@"
fi

require_cmd() {
  name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "ERROR: required command '$name' not found in PATH" >&2
    exit 127
  fi
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
