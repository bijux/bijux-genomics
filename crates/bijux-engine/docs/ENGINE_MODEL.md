# ENGINE_MODEL

## Deterministic Inputs → Deterministic Outputs
Given the same execution graph, inputs, and policy:
- graph hash is stable
- step hashes are stable
- layout tree paths are stable
- manifest hash is stable

### Allowed Nondeterminism
- wall-clock timestamps in execution records
- runtime resource metrics (cpu/mem)

## Purity
Engine performs no execution effects. It only orchestrates:
- no process spawn
- no docker
- no network

This is enforced by `tests/effect_boundary.rs`.
