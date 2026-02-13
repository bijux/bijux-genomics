# vcf_damage_aware_genotype_mini

## Purpose
Demonstrate deterministic coverage-regime selection and visible before/after damage-filter effects for aDNA-style genotype workflows.

Canonical invocation: `./scripts/examples/run.sh vcf_damage_aware_genotype_mini`

## Step 1 Containers
- Resolve container plan with `ensure-images --plan`.

## Step 2 Build/Verify
- Validate profile contract with coverage decision enabled.
- Verify damage-aware stages are present: `vcf.call_gl`, `vcf.damage_filter`, `vcf.call_pseudohaploid`.

## Step 3 Bench
- Execute workflow that branches by coverage regime and applies damage filtering.

## Step 4 Collect/Report
- Collect deterministic outputs under `artifacts/examples/vcf_damage_aware_genotype_mini/`.
- Report regime choice and damage-filter deltas in `report.json`.
