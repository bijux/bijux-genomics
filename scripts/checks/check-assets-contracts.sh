#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
assets = root / "assets"
allowlist = root / "assets" / "LARGE_FILE_ALLOWLIST.txt"
max_mb = 5
max_bytes = max_mb * 1024 * 1024
errors: list[str] = []

allowed_large = set()
if allowlist.exists():
    for raw in allowlist.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        allowed_large.add(line)

def is_doc_file(p: Path) -> bool:
    return p.name in {"README.md", "index.md", "GENERATE.md", "CHECKSUMS.sha256"} or p.suffix in {".md", ".toml", ".yaml", ".yml", ".json"}

# checksum + GENERATE for toy/golden packages
toy_packages = [d for d in sorted((assets / "toy").glob("*")) if d.is_dir()]
golden_packages = [d.parent for d in sorted((assets / "golden").rglob("GENERATE.md"))]
for d in toy_packages + golden_packages:
    rel = d.relative_to(root)
    checksum = d / "CHECKSUMS.sha256"
    generate = d / "GENERATE.md"
    if not checksum.exists():
        errors.append(f"{rel}: missing CHECKSUMS.sha256")
    if not generate.exists():
        errors.append(f"{rel}: missing GENERATE.md")
    else:
        text = generate.read_text(encoding="utf-8")
        for heading in ("## Command(s)", "## Tool versions", "## Input origins", "## Expected outputs"):
            if heading not in text:
                errors.append(f"{rel}/GENERATE.md: missing section '{heading}'")

# large file policy
for f in sorted(assets.rglob("*")):
    if not f.is_file():
        continue
    size = f.stat().st_size
    if size <= max_bytes:
        continue
    rel = str(f.relative_to(root))
    if rel not in allowed_large:
        errors.append(f"{rel}: exceeds {max_mb}MB and is not in assets/LARGE_FILE_ALLOWLIST.txt")

# filename normalization for toy/golden data files
allowed_names = {
    "reads_1.fastq",
    "reads_2.fastq",
    "reads.fastq",
    "toy.vcf",
    "toy.sam",
}
for d in [assets / "toy", assets / "golden"]:
    for f in sorted(d.rglob("*")):
        if not f.is_file():
            continue
        if f.name in {"README.md", "index.md", "GENERATE.md", "CHECKSUMS.sha256", "manifest.json", "metrics.json", "artifact_checksums.json", "report.html"}:
            continue
        if f.suffix in {".md", ".toml", ".yaml", ".yml", ".json"}:
            continue
        if f.name not in allowed_names:
            errors.append(f"{f.relative_to(root)}: filename not normalized (allowed data names: {sorted(allowed_names)})")

# provenance footer in assets index/readme files
for md in sorted(assets.rglob("*.md")):
    if md.name not in {"README.md", "index.md"}:
        continue
    text = md.read_text(encoding="utf-8")
    if "Asset Provenance Footer" not in text:
        errors.append(f"{md.relative_to(root)}: missing 'Asset Provenance Footer'")
    if "Last regenerated:" not in text:
        errors.append(f"{md.relative_to(root)}: missing 'Last regenerated:' line")
    if "Regenerate command:" not in text:
        errors.append(f"{md.relative_to(root)}: missing 'Regenerate command:' line")

if errors:
    print("assets-contracts: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("assets-contracts: OK")
PY
