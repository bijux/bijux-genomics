# VCF Imputation Contract (Species-Aware)

Schema version: `bijux.vcf.imputation_contract.v1`

Authority: `domain/vcf/*` YAML remains the source of truth. This document defines strict runtime/planner contracts for downstream VCF imputation.

## Supported Input Regimes (Enforced)

1. `lowcov_gl`
- Description: low-coverage aDNA or low-coverage cohorts with GL/PL/GP-style likelihood fields.
- Required entry fields: at least one of `GL`, `PL`, `GP`.

2. `diploid_gt`
- Description: low/medium/high coverage diploid GT workflow.
- Required entry fields: diploid-compatible `GT`.

3. `pseudohaploid_to_diploid`
- Status: **refused** in v1.
- Rationale: statistically meaningful pseudo-haploid to diploid expansion requires explicit model and calibration not yet stabilized in production contracts.
- Behavior: hard fail if requested downstream stages require diploid phasing/imputation.

## SpeciesContext Contract (Required Planner Input)

Produced by `bijux-dna-db-ref`, consumed by planner and stage validators.

Required fields:
- `species_id`: canonical species key
- `build_id`: canonical reference build key
- `contig_set_digest`: digest of contig names + lengths
- `contigs`: ordered contig list with lengths
- `sex_system`: one of `{xy, zw, haplodiploid, unknown}`
- `par_policy`: PAR interpretation or `unsupported`

Optional fields:
- `default_coverage_regime`

## Entry VCF Invariants (Keyed to SpeciesContext)

Hard requirements:
- build matches `SpeciesContext.build_id`
- contig digest matches `SpeciesContext.contig_set_digest`
- sorted by contig/position
- bgzip + tabix index present when required by downstream stage
- non-empty and unique sample IDs
- ploidy constraints compatible with selected regime and sex/PAR policy

## Panel + Map Invariants (Keyed to `{species_id, build_id}`)

Hard requirements:
- panel/map species and build match `SpeciesContext`
- contig set/digest match
- required format/index constraints are satisfied
- panel sample count meets minimum
- license metadata present and permitted
- checksum locks present and matching

## Uniform Refusal Rules (Hard Fail)

- build mismatch
- contig mismatch
- overlap below configured minimum
- unsupported sex system/PAR policy for requested operation
- missing required fields for selected backend (for example GL for GL-based backend)

## Stage IO + Artifact Contracts

### `vcf.prepare_reference_panel`
- Inputs: panel VCF/BCF + index, species/build metadata
- Outputs: normalized panel VCF/BCF + index
- Required artifacts:
  - `panel_overlap_stats.json`
  - `panel_lock_resolution.json`
  - `provenance.json`
  - `checksums.sha256`
  - `logs.txt`

### `vcf.phasing`
- Inputs: filtered VCF, panel/map (if backend requires), species context
- Outputs: phased VCF + index
- Required artifacts:
  - `phasing_qc.json`
  - `switch_error_proxy.tsv`
  - `provenance.json`
  - `checksums.sha256`
  - `logs.txt`

### `vcf.impute`
- Inputs: phased or GL-compatible VCF, panel/map, species context
- Outputs: imputed VCF + index
- Required artifacts:
  - `imputation_qc.json`
  - `maf_bin_quality.tsv`
  - `overlap_stats.json`
  - `provenance.json`
  - `checksums.sha256`
  - `logs.txt`

### `vcf.postprocess`
- Inputs: imputed VCF + index
- Outputs: normalized final VCF + index (`.vcf.gz` + `.tbi`)
- Required artifacts:
  - `header_normalization_report.json`
  - `filter_counts.tsv`
  - `provenance.json`
  - `checksums.sha256`
  - `logs.txt`

### `vcf.qc`
- Inputs: postprocess/final VCF + index
- Outputs: QC tables/histograms + summary JSON
- Required artifacts:
  - `qc_summary.json`
  - `qc_tables.tsv`
  - `qc_histograms.json`
  - `provenance.json`
  - `checksums.sha256`
  - `logs.txt`

### `decision.imputation_accept`
- Inputs: imputation QC, overlap stats, policy thresholds, regime metadata
- Outputs: acceptance decision record
- Required artifacts:
  - `imputation_accept_decision.json`
  - `imputation_accept_reasons.json`
  - `provenance.json`

## Output Guarantees

- Final canonical output is bgzip+tabix VCF (`.vcf.gz` + `.tbi`)
- Optional BCF output allowed, not authoritative
- Deterministic header normalization is required for stable diffs

## Explainability Contract (`explain.json`)

`explain.json` MUST include explicit rationale for:
- backend selection
- panel selection
- map selection
- chunking strategy
- `decision.imputation_accept` result and reason traces
