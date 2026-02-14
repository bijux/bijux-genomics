#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
python3 - "$ROOT_DIR" <<'PY'
import sys
from pathlib import Path
try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]
root = Path(sys.argv[1])
accept = tomllib.loads((root / "configs/vcf/downstream_acceptance.toml").read_text(encoding="utf-8"))
stages = {str(row.get("stage_id", "")): row.get("acceptance", []) for row in accept.get("stage", [])}
required = ["vcf.prepare_reference_panel", "vcf.phasing", "vcf.impute", "vcf.postprocess"]
errors = []
for sid in required:
    vals = stages.get(sid)
    if not vals:
        errors.append(f"missing stage acceptance criteria for {sid}")
if not accept.get("production_badge", {}).get("require_qc_accept", False):
    errors.append("production_badge.require_qc_accept must be true")

checklist = root / "docs/30-operations/VCF_DOWNSTREAM_READINESS_CHECKLIST.md"
if not checklist.exists():
    errors.append(f"missing checklist doc: {checklist.relative_to(root)}")

if errors:
    print("vcf downstream readiness: FAILED", file=sys.stderr)
    for e in errors:
        print(f" - {e}", file=sys.stderr)
    raise SystemExit(1)
print("vcf downstream readiness: OK")
PY
