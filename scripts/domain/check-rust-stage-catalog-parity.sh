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
import re
import sys

root = Path(sys.argv[1])

def parse_domain_stage_ids(domain: str) -> set[str]:
    index_path = root / "domain" / domain / "index.yaml"
    text = index_path.read_text(encoding="utf-8")
    out = set()
    in_block = False
    for line in text.splitlines():
        if re.match(r"^stage_ids:\s*$", line):
            in_block = True
            continue
        if in_block:
            m = re.match(r"^\s*-\s*([^\s#]+)\s*$", line)
            if m:
                out.add(m.group(1).strip().strip('"'))
                continue
            if line and not line.startswith(" "):
                break
    return out

def parse_stage_catalog(path: Path, const_name: str) -> set[str]:
    text = path.read_text(encoding="utf-8")
    m = re.search(
        rf"pub\s+const\s+{re.escape(const_name)}:\s*&\[\s*&str\s*\]\s*=\s*&\[(.*?)\];",
        text,
        flags=re.DOTALL,
    )
    if not m:
        raise SystemExit(f"missing {const_name} in {path}")
    body = m.group(1)
    return set(re.findall(r'"([a-z0-9_.]+)"', body))

catalog_specs = [
    ("fastq", root / "crates/bijux-dna-domain-fastq/src/id_catalog.rs", "FASTQ_STAGE_ID_CATALOG"),
    ("bam", root / "crates/bijux-dna-domain-bam/src/types/mod.rs", "BAM_STAGE_ID_CATALOG"),
    ("vcf", root / "crates/bijux-dna-domain-vcf/src/lib.rs", "VCF_STAGE_ID_CATALOG"),
]

errors = []
for dom, path, const_name in catalog_specs:
    domain_ids = parse_domain_stage_ids(dom)
    rust_ids = parse_stage_catalog(path, const_name)
    missing = sorted(domain_ids - rust_ids)
    extra = sorted(rust_ids - domain_ids)
    for sid in missing:
        errors.append(f"{path.relative_to(root)}: {const_name} missing domain stage '{sid}'")
    for sid in extra:
        errors.append(f"{path.relative_to(root)}: {const_name} has stale non-domain stage '{sid}'")

if errors:
    print("rust stage catalog parity check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("rust stage catalog parity: OK")
PY
