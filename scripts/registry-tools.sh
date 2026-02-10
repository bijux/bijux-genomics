#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

cmd="${1:-}"
case "$cmd" in
  list-tools)
    cargo run --bin bijux-dna -- registry list-tools
    ;;
  list-stages)
    cargo run --bin bijux-dna -- registry list-stages
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
    # Registry CLI currently does not expose a runtime-filtered list.
    # Keep compatibility by returning the supported tool set in this scope.
    cargo run --bin bijux-dna -- registry list-tools | paste -sd, -
    ;;
  *)
    echo "unknown command: $cmd" >&2
    exit 2
    ;;
esac
