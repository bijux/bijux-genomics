# VCF Imputation Scope

## Purpose
Define what "imputation" means in bijux for VCF workflows and the scientific/operational boundaries.

## Scope
This scope covers planned VCF downstream stages: phasing, impute, postprocess, and reference-panel preparation.

## Non-goals
- Declaring all imputation tools production-ready.
- Replacing stage-level contracts in `domain/vcf/stages/*.yaml`.

## Imputation In Bijux
Imputation in bijux means: converting partially observed genotypes into inferred genotypes using a pinned reference panel and a pinned phasing/imputation toolchain, with reproducible command contracts and artifacts.

## Supported Input Formats
- VCF/BCF (preferred canonical exchange format).
- PLINK sets (`.bed/.bim/.fam`) when required by specific tools.
- Reference panel artifacts explicitly versioned and pinned.

## aDNA Constraints
- Low coverage and pseudo-haploid samples must be treated as constrained inputs.
- Caller/model assumptions can bias downstream imputation; defaults must declare those assumptions.
- Phasing/imputation comparisons are valid only within pinned references and fixed preprocessing.

## Modern DNA Constraints
- Diploid assumptions are common and must be explicit in stage defaults.
- Cohort composition and ancestry mismatch can change imputation quality; this is a documented failure mode.
- Deterministic runs require pinned versions, stable references, and isolated output roots.

## Contracts
- Every planned imputation stage must have domain stage YAML + fixture coverage.
- Tool admission follows `docs/50-reference/TOOL_ADMISSION.md` and container policy gates.
- Uncontainerized tools remain explicitly external until promoted.

## Failure Modes
- Reference panel mismatch with target build produces invalid outputs.
- Unpinned tool/reference versions break comparability.
- Running outside isolate can leak outputs and violate reproducibility guarantees.
