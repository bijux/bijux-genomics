#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

CONFIG_PATH="${ROOT_DIR}/configs/runtime/execution_kernel.toml"

if [[ ! -f "${CONFIG_PATH}" ]]; then
  echo "check-runtime-execution-kernel-config: missing ${CONFIG_PATH}" >&2
  exit 1
fi

python3 - "${CONFIG_PATH}" <<'PY'
import sys
from pathlib import Path

try:
    import tomllib  # py3.11+
    def loads(raw: str):
        return tomllib.loads(raw)
except Exception:
    try:
        import tomli
        def loads(raw: str):
            return tomli.loads(raw)
    except Exception:
        try:
            import toml
            def loads(raw: str):
                return toml.loads(raw)
        except Exception as exc:  # pragma: no cover
            raise SystemExit(f"python toml parser unavailable (need tomllib/tomli/toml): {exc}")

path = Path(sys.argv[1])
data = loads(path.read_text(encoding="utf-8"))

def err(msg: str) -> None:
    raise SystemExit(f"check-runtime-execution-kernel-config: {msg}")

def is_tmp_root(value: str) -> bool:
    return value == "/tmp" or value == "/var/tmp" or value.startswith("/tmp/") or value.startswith("/var/tmp/")

def check_positive(name: str, value, integer=True):
    if value is None:
        return
    if integer and not isinstance(value, int):
        err(f"{name} must be an integer")
    if value <= 0:
        err(f"{name} must be > 0")

check_positive("default_threads", data.get("default_threads"))
check_positive("default_memory_mb", data.get("default_memory_mb"))
check_positive("default_compression_threads", data.get("default_compression_threads"))
check_positive("default_timeout_s", data.get("default_timeout_s"))
check_positive("max_local_heavy_parallel", data.get("max_local_heavy_parallel"))
check_positive("bgzip_tabix_max_parallel", data.get("bgzip_tabix_max_parallel"))

for key in ("default_temp_root", "cache_root"):
    value = data.get(key)
    if value is None:
        continue
    if not isinstance(value, str) or not value.strip():
        err(f"{key} must be a non-empty string")
    if is_tmp_root(value):
        err(f"{key} cannot point to system tmp ({value})")

patterns = data.get("heavy_stage_patterns")
if patterns is not None:
    if not isinstance(patterns, list):
        err("heavy_stage_patterns must be an array")
    for i, pattern in enumerate(patterns):
        if not isinstance(pattern, str) or not pattern.strip():
            err(f"heavy_stage_patterns[{i}] must be a non-empty string")

per_stage = data.get("per_stage", {})
if per_stage is None:
    per_stage = {}
if not isinstance(per_stage, dict):
    err("per_stage must be a table")
for pattern, knobs in per_stage.items():
    if not isinstance(pattern, str) or not pattern.strip():
        err("per_stage has an empty pattern key")
    if not isinstance(knobs, dict):
        err(f"per_stage.{pattern} must be a table")
    check_positive(f"per_stage.{pattern}.threads", knobs.get("threads"))
    check_positive(f"per_stage.{pattern}.memory_mb", knobs.get("memory_mb"))
    check_positive(f"per_stage.{pattern}.compression_threads", knobs.get("compression_threads"))
    check_positive(f"per_stage.{pattern}.timeout_s", knobs.get("timeout_s"))
    temp_root = knobs.get("temp_root")
    if temp_root is not None:
        if not isinstance(temp_root, str) or not temp_root.strip():
            err(f"per_stage.{pattern}.temp_root must be a non-empty string")
        if is_tmp_root(temp_root):
            err(f"per_stage.{pattern}.temp_root cannot point to system tmp ({temp_root})")

print("check-runtime-execution-kernel-config: OK")
PY
