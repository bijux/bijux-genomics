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
    cargo run --bin bijux -- dna registry list-tools
    ;;
  list-stages)
    cargo run --bin bijux -- dna registry list-stages
    ;;
  show-tool)
    tool_id="${2:-}"
    if [ -z "$tool_id" ]; then
      echo "usage: $0 show-tool <tool-id>" >&2
      exit 2
    fi
    cargo run --bin bijux -- dna registry show-tool "$tool_id"
    ;;
  stage-tools)
    stage_id="${2:-}"
    kind="${3:-all}"
    if [ -z "$stage_id" ]; then
      echo "usage: $0 stage-tools <stage-id> [all|primary|optional|validation|reporting]" >&2
      exit 2
    fi
    cargo run --bin bijux -- dna registry list-tools --stage "$stage_id" --kind "$kind"
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
    python3 - "$ROOT_DIR" "$runtime" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
runtime = sys.argv[2]
registry_files = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
tools = set()
for reg in registry_files:
    if not reg.exists():
        continue
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        tool_id = row.get("id") or row.get("tool_id")
        runtimes = row.get("runtimes", [])
        if not (tool_id and isinstance(runtimes, list) and runtime in runtimes):
            continue
        # Include all registry rows for the runtime when a concrete runtime definition exists.
        # This keeps planned downstream tools with real container defs in scope while excluding
        # placeholders that cannot be built (e.g., missing .def/.Dockerfile).
        if runtime == "apptainer":
            def_rel = str(row.get("apptainer_def", "")).strip()
            if not def_rel:
                continue
            if not (root / def_rel).exists():
                continue
        elif runtime == "docker":
            docker_rel = str(row.get("dockerfile", "")).strip()
            if not docker_rel:
                continue
            if not (root / docker_rel).exists():
                continue
        tools.add(str(tool_id))
print(",".join(sorted(tools)))
PY
    ;;
  *)
    echo "unknown command: $cmd" >&2
    exit 2
    ;;
esac
