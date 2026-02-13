# vcf_downstream_vcf_full_mini

## Purpose
Run a full downstream VCF workflow from toy VCF inputs through PCA, ROH, IBD, and demography with deterministic golden outputs.

Canonical invocation: `./scripts/examples/run.sh vcf_downstream_vcf_full_mini`

## Step 1 Containers
- Ensure container plan is resolved by the runner (`ensure-images --plan`).

## Step 2 Build/Verify
- Validate `example.toml` contract and corpus `corpus-01-mini`.
- Confirm stage contract ordering: `population_structure -> roh -> ibd -> demography`.

## Step 3 Bench
- Run downstream stages on toy VCF:
  - `vcf.population_structure`
  - `vcf.roh`
  - `vcf.ibd`
  - `vcf.demography`

## Step 4 Collect/Report
- Collect deterministic outputs under `artifacts/examples/vcf_downstream_vcf_full_mini/`.
- Emit bundle, `plan.json`, `explain.json`, and `report.json`.
