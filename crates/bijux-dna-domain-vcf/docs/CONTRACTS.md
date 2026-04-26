# bijux-dna-domain-vcf Contracts

This crate is the VCF domain contract source for the workspace. It defines typed VCF stage,
parameter, metric, taxonomy, invariant, and generated-registry materialization contracts.

## Owned Contracts

- Stage IDs for canonical call/filter/stats stages and downstream VCF analysis stages.
- Typed params for VCF calling, filtering, stats, genotype likelihoods, diploid calling,
  pseudohaploid calling, damage filtering, and GL propagation.
- Metrics schemas for call summaries, filter breakdowns, and VCF stats.
- Stage IO, stage metrics, stage delivery, panel governance, and invariant contracts.
- Coverage reports that mark contract-vs-execution readiness.
- Deterministic TOML materialization for VCF param registry and required tool registry views.

## Change Rules

- Adding a public param or metric schema requires updates to public catalogs and contract tests.
- Changing generated TOML output requires updating the committed generated config artifact in the
  same logical change.
- Stage taxonomy or downstream order changes require transition tests.
- Invariant or panel governance changes require explicit failure-mode tests.

## Failure Patterns

- Schema drift between public catalogs and generated registries.
- Unsupported downstream stage transitions.
- VCF files that are unsorted, not bgzip-compressed, missing tabix indexes, or inconsistent across
  sample and contig sets.
- Panel governance records with invalid checksums, incompatible reference builds, or disallowed
  license constraints.
