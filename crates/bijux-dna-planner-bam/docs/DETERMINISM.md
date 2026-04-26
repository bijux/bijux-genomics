# Determinism

Planning must be deterministic for the same profile, inputs, repository tool registry, and feature set.

## Stable Inputs
- Pipeline ID.
- Ordered profile stages.
- Selected tool IDs.
- Stage params and overrides.
- Contract hashes.
- Plan policy.
- Input and output path templates.

## Required Practices
- Use sorted stage and tool collections at public boundaries.
- Prefer deterministic upstream profile ordering.
- Keep graph edges derived from ordered stage plans.
- Keep explain payload fields stable enough for snapshots.

## Determinism Breakers
- Unordered map iteration in output payloads.
- Tool selection that depends on filesystem or environment discovery.
- Runtime probing.
- Process execution.
- Network access.

## Enforcement
- `tests/determinism.rs` covers plan ordering and stable graph behavior.
- `tests/contracts/plan/*` and `tests/contracts/graph/*` snapshot command and graph payloads.
