#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

out_root="${ARTIFACT_DIR:-$ROOT_DIR/artifacts}/certification"
mkdir -p "$out_root"

echo "[cert] FASTQ smoke"
"$ROOT_DIR/scripts/run.sh" smoke run fastq
echo "[cert] BAM smoke"
"$ROOT_DIR/scripts/run.sh" smoke run bam
echo "[cert] VCF downstream mini validation"
"$ROOT_DIR/scripts/run.sh" tooling validate-frontend-mini-domain-stacks

bundle="$out_root/certification_bundle.json"
python3 - "$bundle" <<'PY'
import json,sys,datetime
payload = {
  "schema_version": "bijux.certification_bundle.v1",
  "generated_at_utc": datetime.datetime.utcnow().replace(microsecond=0).isoformat()+"Z",
  "suite": ["smoke_fastq", "smoke_bam", "validate_frontend_mini_domain_stacks"],
  "status": "ok",
}
with open(sys.argv[1], "w", encoding="utf-8") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
echo "certification-gate: OK ($bundle)"
