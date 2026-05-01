# Reference and External Data Scenarios

This operation exercises iteration-18 goals G171-G180 through governed scenario evaluators.

## Command

```bash
cargo run -q -p bijux-dna-dev -- tooling run reference-external-data
```

Optional filters:

```bash
cargo run -q -p bijux-dna-dev -- tooling run reference-external-data -- --scenario G171
cargo run -q -p bijux-dna-dev -- tooling run reference-external-data -- --scenario g179_ena_batch_accession
cargo run -q -p bijux-dna-dev -- tooling run reference-external-data -- --out artifacts/reference_external_data/custom.json
```

## Output

- Default report path: `artifacts/reference_external_data/scenario_suite.json`
- Each row records `goal_id`, `scenario_id`, `status`, scenario notes, and structured evidence.

## Covered Goals

- `G171` `g171_canfam4_reference`: non-human CanFam4 contract resolution.
- `G172` `g172_grch_human_reference`: GRCh38 reference + panel/map/tool compatibility.
- `G173` `g173_bacterial_reference`: microbial alignment/QC path with taxonomy-advisory caveats.
- `G174` `g174_organellar_reference`: organellar and PAR-support evidence.
- `G175` `g175_multi_reference_refusal`: cross-species/cross-build refusal examples.
- `G176` `g176_reference_update_impact`: reference drift invalidation surface report.
- `G177` `g177_contaminant_update_impact`: contaminant-source drift impact report.
- `G178` `g178_adapter_primer_update_impact`: adapter/primer drift impact report.
- `G179` `g179_ena_batch_accession`: ENA batch accession conversion with uncertainty propagation.
- `G180` `g180_offline_data_package`: offline reference/data package materialization report.

## Scope
This document defines the operational or architecture surface for this workflow surface.

## Non-goals
- Replacing crate-level implementation details or test contracts.

## Contracts
- Changes to this surface must stay aligned with governed checks and generated references.

## Purpose
This document records the durable intent and enforcement boundary for this surface.
