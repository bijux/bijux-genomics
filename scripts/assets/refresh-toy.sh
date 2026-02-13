#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STAGE_DIR="${ROOT}/artifacts/tmp/assets/toy/core-v1"
TARGET_DIR="${ROOT}/assets/toy/core-v1"

rm -rf "${STAGE_DIR}"
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

rm -rf "${TARGET_DIR}"
mkdir -p "$(dirname "${TARGET_DIR}")"
cp -R "${STAGE_DIR}" "${TARGET_DIR}"
echo "toy refresh: wrote ${TARGET_DIR}"
