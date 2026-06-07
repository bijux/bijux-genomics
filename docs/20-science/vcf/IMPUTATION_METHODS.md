# VCF Imputation Methods

## Purpose
Provide reference-grade method defaults for planned VCF imputation workflows in bijux.

## Scope
Covers two recommended paths:
- aDNA low-coverage (GL-based style; GLIMPSE-like workflow).
- Modern diploid cohorts (phasing + imputation; SHAPEIT5/IMPUTE5-like workflow).
Both paths stay inside the governed `vcf.prepare_reference_panel` -> `vcf.phasing` -> `vcf.impute` -> `vcf.imputation_metrics` -> `vcf.postprocess` family.

## Non-goals
- Declaring scientific equivalence between tools.
- Replacing stage/tool contracts in `domain/vcf/`.

## aDNA Low Coverage Defaults
- Input model: genotype likelihoods / uncertain calls from low-depth data.
- Preferred flow:
  1. `vcf.prepare_reference_panel` for panel normalization.
  2. `vcf.phasing` with conservative assumptions when a backend requires phased input.
  3. `vcf.imputation_metrics` to keep the admitted GL-oriented metrics surface explicit.
  4. `vcf.impute` with the chosen GL-oriented backend (GLIMPSE-style baseline family).
  5. `vcf.postprocess` for INFO/filter normalization.
- Practical defaults:
  - smaller chunk windows for memory control,
  - explicit random seed,
  - conservative quality filtering in postprocess.

## Modern Diploid Defaults
- Input model: diploid genotype calls with cohort-scale references.
- Preferred flow:
  1. `vcf.prepare_reference_panel`
  2. `vcf.phasing` (SHAPEIT5/Eagle family)
  3. `vcf.imputation_metrics` to keep the admitted Beagle/IMPUTE5/Minimac4 metrics surface explicit
  4. `vcf.impute` (IMPUTE5/Minimac4/Beagle family)
  5. `vcf.postprocess`
- Practical defaults:
  - deterministic seed and fixed threads,
  - stable chunk size and output format,
  - fixed reference panel/build metadata.

## Validity Limits
- Method defaults are valid only under pinned tool/reference versions.
- Cross-tool comparisons are not portable unless preprocessing and panel are fixed.
- aDNA pseudo-haploid inputs can violate diploid assumptions in modern pipelines.
- Planned wrapper images are QA/contract placeholders; full scientific runs require promoted containers.

## Contracts
- Tool admission requirements: `docs/50-reference/TOOL_ADMISSION.md`.
- Stage contracts: `domain/vcf/stages/*.yaml`.
- Scope constraints: `docs/20-science/vcf/IMPUTATION_SCOPE.md`.
