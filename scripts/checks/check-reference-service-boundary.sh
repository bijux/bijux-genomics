#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo "Usage: scripts/checks/check-reference-service-boundary.sh"
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
from datetime import date
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
scan_roots = [
    root / "crates/bijux-dna-api/src/internal/handlers",
    root / "crates/bijux-dna-api/src/run_sections",
    root / "crates/bijux-dna-stages-vcf/src",
    root / "crates/bijux-dna-stages-bam/src",
    root / "crates/bijux-dna-stages-fastq/src",
]

ref_usage = re.compile(
    r"reference_fasta|reference_path|reference\\s*=|fasta|\\.fai\\b|mt_reference|genome_build|build_id",
    re.IGNORECASE,
)
direct_path = re.compile(
    r"Path::new\(|PathBuf::from\(|std::fs::read_to_string\(|std::fs::metadata\(",
)
service_markers = (
    "ref_service(",
    "resolve_reference_bundle(",
    "resolve_reference_bank(",
    "resolve_species_authority(",
    "resolve_contig_map(",
    "enforce_declared_build_and_contigs(",
    "resolve_default_reference_set(",
)
file_name_scope = re.compile(r"(executor|handler|runtime|orchestration|entrypoint|pipeline)", re.IGNORECASE)

violations = []
for base in scan_roots:
    for file in base.rglob("*.rs"):
        text = file.read_text(encoding="utf-8")
        rel = str(file.relative_to(root))
        if not file_name_scope.search(rel):
            continue
        if not ref_usage.search(text):
            continue
        if not direct_path.search(text):
            continue
        if any(marker in text for marker in service_markers):
            continue
        violations.append(rel)

allow_cfg = root / "configs/ci/registry/reference_service_boundary_allowlist.toml"
allow_paths = set()
if allow_cfg.exists():
    data = tomllib.loads(allow_cfg.read_text(encoding="utf-8"))
    for idx, row in enumerate(data.get("allow", []), 1):
        path = row.get("path")
        expires_on = row.get("expires_on")
        if not isinstance(path, str) or not path.strip():
            raise SystemExit(f"reference-service-boundary: invalid allowlist path in entry #{idx}")
        if not isinstance(expires_on, str):
            raise SystemExit(f"reference-service-boundary: missing expires_on for {path}")
        try:
            expiry = date.fromisoformat(expires_on)
        except Exception as exc:
            raise SystemExit(f"reference-service-boundary: invalid expires_on `{expires_on}` for {path}: {exc}") from exc
        if expiry < date.today():
            raise SystemExit(f"reference-service-boundary: expired allowlist entry for {path} (expired {expires_on})")
        allow_paths.add(path)

effective = sorted(v for v in violations if v not in allow_paths)

if effective:
    print("reference-service-boundary: FAILED", file=sys.stderr)
    print("Executors touching reference paths must resolve via bijux-dna-db-ref service API.", file=sys.stderr)
    for item in effective:
        print(f"- {item}", file=sys.stderr)
    raise SystemExit(1)

print("reference-service-boundary: OK")
PY
