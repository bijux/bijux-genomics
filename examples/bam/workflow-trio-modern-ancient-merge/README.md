# bam_workflow_trio_modern_ancient_merge

## Purpose
Provide one fixture-safe BAM trio package that demonstrates three governed surfaces in a single runnable example contract:
- modern WGS QC and mapping evidence
- ancient-DNA damage/authenticity/contamination advisory evidence
- lane merge compatibility plus downstream coverage lineage checks

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run bam_workflow_trio_modern_ancient_merge`

## Step 1 Containers
- Resolve image planning for BAM validate, mapping summary, markdup, coverage, damage, authenticity, and contamination tools.

## Step 2 Build/Verify
- Validate `corpus-01-mini` availability and reference bundle identity.
- Validate the trio track contracts in `workflow-manifest.json`.
- Validate merge/coverage lineage evidence is preserved as explicit artifact contracts and never inferred silently.

## Step 3 Bench
- Execute the three fixture-safe tracks in simulation mode:
  - `modern_wgs_qc`
  - `ancient_damage_contamination`
  - `merge_coverage_lineage`

## Step 4 Collect/Report
- Collect outputs under `artifacts/examples/bam_workflow_trio_modern_ancient_merge/`.
- Preserve advisory boundaries for damage/authenticity/contamination and keep merge compatibility evidence independent from coverage interpretation.

## Contracts
- `example.toml` is the runnable contract.
- `tiny-inputs.json` pins fixture-safe inputs for the three BAM tracks.
- `workflow-manifest.json` defines stage order per track.
- `expected-evidence.json` defines required BAM evidence outputs for review.
