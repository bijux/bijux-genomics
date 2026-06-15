# VCF Imputation Scope

## Purpose
Define what "imputation" means in bijux for VCF workflows and the scientific/operational boundaries.

## Scope
This scope covers the governed imputation family:
- `vcf.prepare_reference_panel` for panel normalization and admission.
- `vcf.phasing` for haplotype preparation before downstream inference.
- `vcf.imputation_metrics` as the admitted imputation-metrics surface.
- `vcf.impute` for explicit panel-backed imputation execution.
- `vcf.postprocess` for deterministic normalization after imputation.
- `vcf.qc` for downstream acceptance summaries on final outputs.
Planning is species/build-aware via `bijux-dna-db-ref` resolution (`resolve_species_context`, `resolve_reference_bundle`).

## Non-goals
- Declaring all imputation tools production-ready.
- Replacing stage-level contracts in `domain/vcf/stages/*.yaml`.

## Imputation In Bijux
Imputation in bijux means: converting partially observed genotypes into inferred genotypes using a pinned reference panel and a pinned phasing/imputation toolchain, with reproducible command contracts and artifacts.

## Supported Input Formats
- VCF/BCF (preferred canonical exchange format).
- PLINK sets (`.bed/.bim/.fam`) when required by specific tools.
- Reference panel artifacts explicitly versioned and pinned.

## Species/Build Governance
- Planner admission requires successful `{species_id, build_id}` resolution to a canonical `SpeciesContext`.
- Planner admission requires successful canonical reference bundle resolution with lock hashes.
- Bundle drift is lock-governed: when `configs/runtime/reference_bundles.toml` changes, `configs/runtime/reference_bundles_lock.sha256` must update or gates fail.
- Supported feature flags are keyed by `{species_id, build_id}` (for example `imputation`, `sex_chr`).
- Contig normalization is policy-governed per bundle:
  - `strict_only`: chr/no-chr mismatches are refused.
  - `deterministic_remap`: remap table is explicit and audited as first-class contract metadata.

## aDNA Constraints
- Low coverage and pseudo-haploid samples must be treated as constrained inputs.
- Caller/model assumptions can bias downstream imputation; defaults must declare those assumptions.
- Phasing/imputation comparisons are valid only within pinned references and fixed preprocessing.
- Pseudo-haploid to diploid imputation remains a hard refusal in v1 contract mode.

## Modern DNA Constraints
- Diploid assumptions are common and must be explicit in stage defaults.
- Cohort composition and ancestry mismatch can change imputation quality; this is a documented failure mode.
- Deterministic runs require pinned versions, stable references, and isolated output roots.

## Ancestry Matching Guidance
- `population_set` in panel metadata must match expected cohort ancestry composition.
- `genome_build` must match input build exactly; no implicit liftover.
- `variant_set_compatibility` must be validated before run admission.

## Contracts
- Every stage in the `vcf.prepare_reference_panel` -> `vcf.phasing` -> `vcf.impute` -> `vcf.postprocess` -> `vcf.qc` chain must have domain stage YAML + fixture coverage.
- `vcf.imputation_metrics` exists as the admitted metrics contract that keeps comparative tool admission explicit while `vcf.impute` stays the executable stage boundary.
- Tool admission follows `docs/50-reference/TOOL_ADMISSION.md` and container policy gates.
- Uncontainerized tools remain explicitly external until promoted.

## Failure Modes
- Reference panel mismatch with target build produces invalid outputs.
- Unpinned tool/reference versions break comparability.
- Running outside isolate can leak outputs and violate reproducibility guarantees.
- Species/build resolution failure blocks planner admission.
- Non-VCF domains (for example eDNA/pollen) are refused by the VCF imputation planner.
