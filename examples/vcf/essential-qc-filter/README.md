# vcf_essential_qc_filter

## Purpose
Run the essential governed VCF validation, filtering, normalization, QC, and stats path with deterministic mini outputs.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run -- vcf_essential_qc_filter`

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
- Collect deterministic outputs under `artifacts/examples/vcf_essential_qc_filter/`.
- Preserve the preflight validation summary, filter explainability artifact, normalization contract, QC report, and stats report for downstream evidence inspection.

## Canonical Contracts
- `example.toml` is the runnable manifest.
- `tiny-inputs.json` records the mini corpus contract used by this example.
- `workflow-manifest.json` records the governed stage order and operating mode.
- `golden/plan.json` is the expected plan contract.
- `expected-evidence.json` records the VCF evidence artifacts that must survive bundling.
