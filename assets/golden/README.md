# Golden Dataset Pack (Tiny)

This pack provides tiny inputs for local smoke runs.

## FASTQ
- Single-end: `assets/golden/smoke-inputs-v1/fastq/se/reads.fastq`
- Paired-end: `assets/golden/smoke-inputs-v1/fastq/pe/reads_1.fastq`, `assets/golden/smoke-inputs-v1/fastq/pe/reads_2.fastq`

## BAM
- Source SAM: `assets/golden/smoke-inputs-v1/bam/sample.sam`
- Generate BAM (requires samtools):
  - `samtools view -bS assets/golden/smoke-inputs-v1/bam/sample.sam > assets/golden/smoke-inputs-v1/bam/sample.bam`
  - `samtools index assets/golden/smoke-inputs-v1/bam/sample.bam`

The BAM is intentionally minimal (single alignment) to keep smoke runs fast.

## Deterministic Toy Runs
- Toy inputs live under `assets/toy/core-v1/*` with checksums in `assets/toy/core-v1/CHECKSUMS.sha256`.
- Golden toy run outputs live under `assets/golden/toy-runs-v1/*` and include:
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
