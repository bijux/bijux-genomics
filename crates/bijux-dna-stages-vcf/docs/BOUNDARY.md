# Boundary

`bijux-dna-stages-vcf` owns VCF stage execution helpers and stage artifacts. It
is allowed to be effectful because VCF stage work currently lives here.

## Owned Here

- VCF stage request and result types.
- Typed VCF stage runners and dispatch helpers.
- VCF preflight invariants and normalized preflight artifacts.
- VCF IO helpers for validation, normalization, bgzip/tabix indexing, region
  extraction, concatenation, checksums, reference matching, and panel overlap.
- Metrics parsers and summaries for VCF call, filter, and stats artifacts.
- Wrapper version checks for declared local VCF tools.
- Stage manifests, runtime explanations, sidecars, checksums, and refusal codes
  produced by VCF stage execution.

## Owned Elsewhere

- CLI commands and user-facing command routing belong in command/API crates.
- Planner policy, profile selection, and cross-domain plan construction belong
  in planner and pipeline crates.
- Runtime queueing, scheduling, retries, cancellation, and worker supervision
  belong in runtime and runner crates.
- Environment provisioning, container image resolution, and installation belong
  in environment crates.
- Domain ID authority belongs in `bijux-dna-domain-vcf`.

## Boundary Risks

- Adding planner, runtime, runner, API, or environment dependencies to this
  crate collapses ownership boundaries.
- Adding new stage IDs in source without documenting the command and stage
  contract surface makes downstream command inventory incomplete.
- Hidden network access breaks reproducibility and must remain outside default
  production paths.
