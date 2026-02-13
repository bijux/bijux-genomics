# vcf_downstream_demography_mini

## Purpose
Run a deterministic mini downstream VCF demography flow using `examples/data/corpus-01-mini`.

Canonical invocation: `./scripts/examples/run.sh vcf_downstream_demography_mini`

## Step 1 Containers
- Ensure image plan is resolved by the runner (`ensure-images --plan`).

## Step 2 Build/Verify
- Validate the example contract and corpus selection (`corpus-01-mini`).

## Step 3 Bench
- Execute downstream stages:
  - `vcf.population_structure`
  - `vcf.roh`
  - `vcf.ibd`
  - `vcf.demography`

## Step 4 Collect/Report
- Collect outputs under `artifacts/examples/vcf_downstream_demography_mini/`.
- Emit deterministic bundle and report artifacts.
