#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STAGE_DIR="${ROOT}/artifacts/assets-refresh/toy/core-v1"
TARGET_DIR="${ROOT}/assets/toy/core-v1"
REPORT_DIR="${ROOT}/artifacts/assets-refresh/toy"

rm -rf "${STAGE_DIR}"
mkdir -p "${REPORT_DIR}"
mkdir -p "${STAGE_DIR}/fastq" "${STAGE_DIR}/bam" "${STAGE_DIR}/vcf"

cat > "${STAGE_DIR}/fastq/reads_1.fastq" <<'DATA'
@read1/1
ACGTTGCAACGT
+
FFFFFFFFFFFF
@read2/1
TGCATGCATGCA
+
FFFFFFFFFFFF
DATA

cat > "${STAGE_DIR}/fastq/reads_2.fastq" <<'DATA'
@read1/2
ACGTTGCAACGT
+
FFFFFFFFFFFF
@read2/2
TGCATGCATGCA
+
FFFFFFFFFFFF
DATA

cat > "${STAGE_DIR}/bam/toy.sam" <<'DATA'
@HD	VN:1.6	SO:coordinate
@SQ	SN:chr1	LN:1000
read1	0	chr1	1	60	12M	*	0	0	ACGTTGCAACGT	FFFFFFFFFFFF
read2	0	chr1	50	60	12M	*	0	0	TGCATGCATGCA	FFFFFFFFFFFF
DATA

cat > "${STAGE_DIR}/vcf/toy.vcf" <<'DATA'
##fileformat=VCFv4.2
##contig=<ID=chr1,length=1000>
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO
chr1	10	.	A	G	60	PASS	.
DATA

(
  cd "${STAGE_DIR}"
  shasum -a 256 bam/toy.sam fastq/reads_1.fastq fastq/reads_2.fastq vcf/toy.vcf > CHECKSUMS.sha256
)

python3 - "$STAGE_DIR" "$REPORT_DIR/report.json" <<'PY'
import hashlib
import json
import subprocess
import sys
from pathlib import Path

stage_dir = Path(sys.argv[1])
report_path = Path(sys.argv[2])
files = sorted([p for p in stage_dir.rglob("*") if p.is_file()])

checksums = {}
for p in files:
    h = hashlib.sha256()
    h.update(p.read_bytes())
    checksums[p.relative_to(stage_dir).as_posix()] = h.hexdigest()

tool_versions = {}
for cmd in [["python3", "--version"], ["shasum", "-a", "256", "--version"]]:
    name = cmd[0]
    try:
        out = subprocess.check_output(cmd, stderr=subprocess.STDOUT, text=True).strip().splitlines()[0]
    except Exception:
        out = "unknown"
    tool_versions[name] = out

report = {
    "schema_version": "bijux.assets.refresh_report.v1",
    "asset": "toy/core-v1",
    "generator_command": "scripts/assets/refresh-toy.sh",
    "inputs": list(checksums.keys()),
    "input_list": list(checksums.keys()),
    "output_checksums": checksums,
    "tool_versions": tool_versions,
    "checksums": checksums,
}
report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"wrote {report_path}")
PY

rm -rf "${TARGET_DIR}"
mkdir -p "$(dirname "${TARGET_DIR}")"
cp -R "${STAGE_DIR}" "${TARGET_DIR}"
echo "toy refresh: wrote ${TARGET_DIR}"
