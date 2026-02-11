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

## Deterministic Toy Runs
- Toy inputs live under `assets/toy/*` with checksums in `assets/toy/CHECKSUMS.sha256`.
- Golden toy run outputs live under `assets/golden/toy_runs/*` and include:
  - `manifest.json`
  - `metrics.json`
  - `report.html`
  - `artifact_checksums.json`

Reproduce:
- `make toy-run-fastq`
- `make toy-run-bam`
- `make toy-run-vcf`
- `make demo`

Validate against goldens:
- `make toy-golden-check`

Refresh goldens (explicit opt-in):
- `make golden-refresh ACCEPT=1`
