# Stage Contracts

This crate currently manages executable contracts for VCF stage families. Each
stage must have an explicit stage ID, typed params when needed, deterministic
artifact paths, and contract tests covering success or refusal behavior.

## Managed Stage IDs

- `vcf.admixture`
- `vcf.call`
- `vcf.call_diploid`
- `vcf.call_gl`
- `vcf.call_pseudohaploid`
- `vcf.damage_filter`
- `vcf.demography`
- `vcf.filter`
- `vcf.gl_propagation`
- `vcf.ibd`
- `vcf.imputation_metrics`
- `vcf.impute`
- `vcf.pca`
- `vcf.phasing`
- `vcf.population_structure`
- `vcf.postprocess`
- `vcf.prepare_reference_panel`
- `vcf.qc`
- `vcf.roh`
- `vcf.stats`

## Artifact Duties

- Stage runners write only under caller-provided output directories.
- Stage runners emit typed metrics, manifests, warnings, sidecars, or readiness
  artifacts appropriate to the stage family.
- Refusal paths must be covered by contract tests when a stage has required
  input regime, reference, panel, map, or metadata preconditions.
- External-tool fallback behavior must remain deterministic for default tests.
