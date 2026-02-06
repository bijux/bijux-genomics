# Golden Dataset Pack (Tiny)

This pack provides tiny inputs for local smoke runs.

## FASTQ
- Single-end: `assets/golden/fastq/se/reads.fastq`
- Paired-end: `assets/golden/fastq/pe/reads_1.fastq`, `assets/golden/fastq/pe/reads_2.fastq`

## BAM
- Source SAM: `assets/golden/bam/sample.sam`
- Generate BAM (requires samtools):
  - `samtools view -bS assets/golden/bam/sample.sam > assets/golden/bam/sample.bam`
  - `samtools index assets/golden/bam/sample.bam`

The BAM is intentionally minimal (single alignment) to keep smoke runs fast.
