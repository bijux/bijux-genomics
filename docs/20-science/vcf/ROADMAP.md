# Damage-Aware Genotype Spec

## Purpose
Define a damage-aware genotype calling contract for VCF workflows, including GL-first and pseudohaploid regimes.

## Scope
`vcf.call_gl`, `vcf.call_diploid`, `vcf.call_pseudohaploid`, `vcf.damage_filter`, and `vcf.gl_propagation` behavior.

## Non-goals
- Claiming one regime is universally optimal across all coverage contexts.
- Replacing sample- and marker-specific interpretation by domain experts.

## Contracts
- Calling regime selection is explicit and versioned in run metadata.
- Damage masking rules (C>T/G>A context and PMD threshold) are explicit and reproducible.
- GL/PL fields must be retained when workflows require likelihood propagation.
- Pseudohaploid outputs are never treated as diploid-equivalent in downstream inference.

## Calling Regimes
- `vcf.call_gl`: emits likelihood-first outputs for low-coverage and aDNA-sensitive workflows.
- `vcf.call_diploid`: emits diploid genotypes for modern high-confidence cohorts.
- `vcf.call_pseudohaploid`: emits one-allele representations for low-coverage contexts where diploid calls are unstable.

## Damage Filter Rules
- Transition-sensitive masking: C>T and G>A contexts can be excluded or down-weighted.
- PMD thresholding: reads/sites above threshold are filtered or annotated per policy.
- All active thresholds must appear in report artifacts and fixture contracts.
- Bias audit must report before/after summary of damage-sensitive signals.

## GL Propagation Rules
- GL/PL retention is mandatory for GL-based downstream stages.
- Sites-only transforms must document whether genotypes were emitted or withheld.
- Any conversion from GL to hard calls must state model and threshold assumptions.

## Why This Exists
- aDNA and other degraded inputs are bias-prone under naive diploid calling.
- Separating regimes avoids silent scientific drift between low-coverage and modern workflows.

## Failure modes
- Mixing pseudohaploid outputs with diploid assumptions in one analysis.
- Dropping GL/PL tags during filtering, making downstream inference invalid.
- Unreported PMD or transition mask changes causing irreproducible results.
- Missing before/after bias audit section in report artifacts.
