# VCF Downstream Roadmap

## Purpose
Document planned downstream VCF stages/tools and why they are not yet in the production baseline.

## Scope
Covers candidate post-calling stages and candidate tool families for VCF analysis expansion.

## Non-goals
- Declaring these tools as production-ready today.
- Replacing domain contracts under `domain/vcf/`.

## Contracts
- Current production baseline remains `bcftools`-centric until stage contracts and fixtures are added.
- New tools enter production only through `docs/50-reference/TOOL_ADMISSION.md` workflow.

## Planned Stages And Candidate Tools
- `vcf.phasing`
  candidates: `beagle`, `shapeit`.
  rationale: required for haplotype-aware downstream analyses.
- `vcf.ld_pruning`
  candidates: `plink`, `plink2`.
  rationale: stable LD-pruned sets are required for PCA/kinship comparability.
- `vcf.population_structure`
  candidates: `eigensoft` (`smartpca`), `plink2`.
  rationale: reproducible principal components and structure summaries.
- `vcf.relatedness`
  candidates: `plink2`, `ibdseq`/IBD-family tools.
  rationale: explicit IBD/kinship inference from VCF-level inputs.
- `vcf.imputation_prep`
  candidates: `beagle`, `bcftools` normalization helpers.
  rationale: normalize and phase before imputation workflows.

## Admission Path
For each stage/tool above:
1. Add stage/tool contracts in `domain/vcf/{stages,tools}`.
2. Add fixtures and default settings rationale.
3. Add registry + container entries with pinned versions.
4. Add contract/snapshot coverage before enabling in production profiles.
