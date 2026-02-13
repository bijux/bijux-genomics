# vcf_imputation_mini

## Purpose
Run a deterministic mini VCF imputation workflow contract using the corpus-01-mini data surface.

Canonical invocation: `./scripts/examples/run.sh vcf_imputation_mini`

## Step 1 Containers
- Ensure image plan is resolved by the runner (`ensure-images --plan`).

## Step 2 Build/Verify
- Validate `example.toml` contract and corpus selection (`corpus-01-mini`).

## Step 3 Bench
- Execute the example-pinned suite for:
  - `vcf.prepare_reference_panel`
  - `vcf.phasing`
  - `vcf.impute`
  - `vcf.postprocess`

## Step 4 Collect/Report
- Collect outputs under `artifacts/examples/vcf_imputation_mini/`.
- Emit bundle and deterministic report files.
