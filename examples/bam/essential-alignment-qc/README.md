# bam_essential_alignment_qc

## Purpose
Run the essential governed FASTQ-to-BAM alignment and QC path with deterministic BAM evidence outputs.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run bam_essential_alignment_qc`

## Step 1 Containers
- Ensure image planning is resolved for the FASTQ preprocessing, reference preparation, BAM alignment, and BAM QC tools.

## Step 2 Build/Verify
- Validate `corpus-01-mini` availability before execution.
- Validate the governed FASTQ-to-BAM stage order:
  - `fastq.preprocess`
  - `core.prepare_reference`
  - `bam.align`
  - `bam.qc_pre`
  - `bam.mapping_summary`
  - `bam.coverage`
  - `bam.damage`
- Validate BAM evidence contracts written during the run:
  - `sample_identity.json`
  - `reference_preflight.json`
  - `alignment_provenance.json`
  - `mapping_summary.json`
  - `coverage.regime.json`
  - `damage.unified_metrics.json`

## Step 3 Bench
- Execute the essential cross-domain alignment handoff with modern BAM defaults and deterministic output layout.

## Step 4 Collect/Report
- Collect governed outputs under `artifacts/examples/bam_essential_alignment_qc/`.
- Preserve the BAM planning, explainability, and report artifacts for downstream evidence inspection.

## Canonical Contracts
- `example.toml` is the runnable manifest.
- `tiny-inputs.json` records the mini corpus and reference contract.
- `workflow-manifest.json` records the governed stage order and operating mode.
- `golden/plan.json` is the expected plan contract.
- `expected-evidence.json` records the BAM evidence artifacts that must survive bundling.
