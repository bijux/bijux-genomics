# vcf_downstream_chr22_reference

## Purpose
Run a downstream VCF workflow from a small target VCF with an explicitly identified chr22 panel and map authority.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run vcf_downstream_chr22_reference`

## Step 1 Containers
- Ensure the container plan is resolved by the runner (`ensure-images --plan`).

## Step 2 Build/Verify
- Validate `example.toml`, the target corpus, and the pinned reference identities.
- Confirm stage contract ordering: `population_structure -> roh -> ibd -> demography`.

## Step 3 Bench
- Run the declared downstream stages on the target VCF.

## Step 4 Collect/Report
- Collect deterministic outputs under `artifacts/examples/vcf_downstream_chr22_reference/`.
- Emit bundle, `plan.json`, `explain.json`, and `report.json`.
