#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

python3 - "$ROOT_DIR" <<'PY'
from __future__ import annotations
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
crates = [
    "bijux-dna-api",
    "bijux-dna-engine",
    "bijux-dna-runner",
    "bijux-dna-stages-bam",
    "bijux-dna-stages-fastq",
    "bijux-dna-stages-vcf",
]
errors = []
for crate in crates:
    crate_dir = root / "crates" / crate
    cargo_toml = crate_dir / "Cargo.toml"
    features_md = crate_dir / "docs" / "FEATURES.md"
    if not features_md.exists():
        errors.append(f"{crate}: missing docs/FEATURES.md")
        continue
    cargo = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    feature_keys = sorted((cargo.get("features") or {}).keys())
    text = features_md.read_text(encoding="utf-8")
    documented = sorted(set(re.findall(r"^`([a-zA-Z0-9_\-]+)`", text, flags=re.M)))
    undocumented = [f for f in feature_keys if f not in documented]
    extra = [f for f in documented if f not in feature_keys]
    if undocumented:
        errors.append(f"{crate}: undocumented features: {', '.join(undocumented)}")
    if extra:
        errors.append(f"{crate}: features listed in docs/FEATURES.md but missing in Cargo.toml: {', '.join(extra)}")

if errors:
    print("ERROR: executor feature docs contract failed", file=sys.stderr)
    for err in errors:
        print(f"  - {err}", file=sys.stderr)
    raise SystemExit(1)
print("check-executor-features-docs: OK")
PY
