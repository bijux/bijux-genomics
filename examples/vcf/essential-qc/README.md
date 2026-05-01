# vcf_essential_qc

## Purpose
Run the essential governed VCF validation, filtering, normalization, QC, and stats path with deterministic mini outputs.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run -- vcf_essential_qc`

## Step 1 Containers
- Ensure image planning is resolved for the VCF filtering, normalization, QC, and stats tools.

## Step 2 Build/Verify
- Validate `corpus-01-mini` availability before execution.
- Validate that VCF preflight enforces header, contig, sample-ID, and reference-context contracts before stage execution.
- Validate the governed essential stage order:
  - `vcf.filter`
  - `vcf.postprocess`
  - `vcf.qc`
  - `vcf.stats`

## Step 3 Bench
- Execute the essential VCF workflow on the mini corpus with governed filter evidence and normalization contracts.

## Step 4 Collect/Report
- Collect deterministic outputs under `artifacts/examples/vcf_essential_qc/`.
- Preserve the preflight validation summary, filter explainability artifact, normalization contract, QC report, and stats report for downstream evidence inspection.
