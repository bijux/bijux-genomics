# bijux-dna-dev Test Taxonomy

Intent buckets in this crate:

- `boundaries`: layering, tree-shape, and ownership guardrails for the development control plane.
- `contracts`: repository automation behavior and generated-output contracts.
- `determinism`: reproducibility checks for generated metadata and repo maintenance flows.
- `schemas`: stable snapshots for automation payloads and governed config surfaces.

Speed model:

- **fast**: crate-local guardrails and deterministic automation checks.
- **slow**: snapshot regeneration or broader workspace validation when explicitly requested.

Entry points:

- `tests/boundaries.rs`: boundary-oriented integration coverage
- `tests/workspace_paths.rs`: shared crate/repository path helpers for integration tests
