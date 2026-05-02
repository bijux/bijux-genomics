# Scientific Caveat Propagation Scenarios

This operation exercises iteration-19 goals G181-G190 through governed caveat-propagation evaluators.

## Command

```bash
cargo run -q -p bijux-dna-dev -- tooling run scientific-caveat-propagation
```

Optional filters:

```bash
cargo run -q -p bijux-dna-dev -- tooling run scientific-caveat-propagation -- --scenario G181
cargo run -q -p bijux-dna-dev -- tooling run scientific-caveat-propagation -- --scenario g190_missing_evidence_propagation
cargo run -q -p bijux-dna-dev -- tooling run scientific-caveat-propagation -- --out artifacts/scientific_caveat_propagation/custom.json
```

## Output

- Default report path: `artifacts/scientific_caveat_propagation/scenario_suite.json`
- Each row records `goal_id`, `scenario_id`, `status`, scenario notes, and structured evidence.

## Covered Goals

- `G181` `g181_ancient_dna_authenticity_caveat_library`: ancient-DNA caveat library from damage/authenticity/contamination/endogenous evidence.
- `G182` `g182_low_pass_genotype_caveat_library`: low-pass caveat library for coverage/GL/imputation/missingness propagation.
- `G183` `g183_edna_taxonomy_caveat_library`: eDNA taxonomy caveats for DB bias, rank resolution, abundance interpretation, and primer sensitivity.
- `G184` `g184_population_structure_caveat_library`: population-structure caveats for sampling bias, LD pruning, cohort size, and label boundaries.
- `G185` `g185_demography_caveat_library`: demography caveats for model assumptions, marker density, and underpowered cohorts.
- `G186` `g186_damage_aware_variant_caveat_library`: damage-aware variant caveat propagation from VCF damage-filter summaries.
- `G187` `g187_contamination_propagation_model`: contamination risk propagation from FASTQ/BAM contamination evidence into VCF/population surfaces.
- `G188` `g188_sample_identity_conflict_propagation`: sample/read-group conflict propagation into downstream refusal boundaries.
- `G189` `g189_reference_build_conflict_propagation`: reference-build mismatch propagation into BAM/VCF/population refusal surfaces.
- `G190` `g190_missing_evidence_propagation`: missing-evidence propagation from evidence-gap diagnostics into final report caveats.

## Purpose
This document describes the governed intent and operator-facing meaning of this surface.

## Scope
The scope is limited to repository-owned behavior, contracts, and evidence paths for this topic.

## Non-goals
This document does not redefine source-of-truth schemas, code ownership boundaries, or release policy outside this surface.

## Contracts
Claims here are valid only when they remain consistent with governed configs, domain authorities, and policy checks.

