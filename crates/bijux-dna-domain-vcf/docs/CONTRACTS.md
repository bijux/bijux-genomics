# bijux-dna-domain-vcf Contracts

This crate is the VCF domain contract source for the workspace. It defines typed VCF stage,
parameter, metric, taxonomy, invariant, and generated-registry materialization contracts.

## Owned Contracts

- Stage IDs for canonical call/filter/stats stages and downstream VCF analysis stages.
- Typed params for VCF calling, filtering, stats, genotype likelihoods, diploid calling,
  pseudohaploid calling, damage filtering, and GL propagation.
- Metrics schemas for call summaries, filter breakdowns, and VCF stats.
- Stage IO, stage metrics, stage delivery, panel governance, and invariant contracts.
- Workflow-surface contracts for validation, artifact classes, reference context, filter evidence,
  normalization semantics, cohort validation, likelihood workflows, phasing/imputation
  boundaries, damage filtering, stats/report coverage, panel boundaries, and population guardrails.
- Typed production corpus manifests for governed VCF regression scenarios.
- Typed scientific-drift reports for defaults, backend, normalization, and filter-policy changes.
- Coverage reports that mark contract-vs-execution readiness.
- Deterministic TOML materialization for VCF param registry and required tool registry views.

## Change Rules

- Adding a public param or metric schema requires updates to public catalogs and contract tests.
- Changing generated TOML output requires updating the committed generated config artifact in the
  same logical change.
- Stage taxonomy or downstream order changes require transition tests.
- Invariant or panel governance changes require explicit failure-mode tests.
- Workflow-surface contract changes require planner, runtime, or example evidence showing where the
  new contract is surfaced to operators.
- Corpus-manifest changes must preserve the governed scenario set or document the reason for the
  new corpus scope.
- Scientific-drift report changes must keep before/after risk reporting explicit and snapshot
  reviewed.

## Failure Patterns

- Schema drift between public catalogs and generated registries.
- Unsupported downstream stage transitions.
- VCF files that are unsorted, not bgzip-compressed, missing tabix indexes, or inconsistent across
  sample and contig sets.
- Panel governance records with invalid checksums, incompatible reference builds, or disallowed
  license constraints.
