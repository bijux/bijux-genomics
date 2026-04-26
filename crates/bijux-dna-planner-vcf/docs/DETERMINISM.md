# Determinism

The VCF planner must produce stable plans for identical inputs, registry files, and reference catalog state.

## Deterministic Inputs
- Stage order comes from `src/stage_sequence.rs` and VCF domain downstream-order validation.
- Tool choices come from explicit overrides or deterministic defaults in `src/tool_catalog.rs`.
- Reference context comes from governed DB-ref catalog views and caller-provided panel locks.
- Param overrides are validated against repository-owned param registries before plans are built.

## Stable Outputs
- Stage plans are emitted in resolved downstream order.
- Graph edges are derived from stage plan order and artifact handoff contracts.
- Explain payloads report selected stages, tools, coverage context, panel locks, and reference decisions deterministically.
- Snapshot tests cover representative coverage regimes and tool overrides.

## Non-Deterministic Behavior Is Forbidden
- Environment probing.
- Runtime tool discovery.
- Process execution.
- Network lookup.
- Wall-clock or random identifiers.
- Iteration over unordered maps when output order is observable.
