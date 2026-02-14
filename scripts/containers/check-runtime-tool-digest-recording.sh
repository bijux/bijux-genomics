#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])
errors = []

stage_file = root / "crates/bijux-dna-stages-vcf/src/pipeline.rs"
stage_text = stage_file.read_text(encoding="utf-8")
for marker in ['"tool_digest": resolve_tool_digest', '"tool_digest": tool_digest']:
    if marker not in stage_text:
        errors.append(f"{stage_file.relative_to(root)} missing marker `{marker}`")

runtime_contract = root / "crates/bijux-dna-runtime/tests/contracts/manifest_integrity.rs"
runtime_text = runtime_contract.read_text(encoding="utf-8")
if "image_digest" not in runtime_text:
    errors.append(f"{runtime_contract.relative_to(root)} missing image_digest contract checks")

if errors:
    print("runtime tool digest recording: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("runtime tool digest recording: OK")
PY
