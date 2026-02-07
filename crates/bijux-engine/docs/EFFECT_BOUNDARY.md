# EFFECT_BOUNDARY

## Rule
The engine must not perform side effects directly. It must not:
- spawn processes
- invoke docker or container runtimes
- access the network
- read or write ad-hoc files outside the run layout

All effects are executed by `Runner` implementations and runtime helpers.

## Enforced by
- `tests/contracts/effect_boundary.rs`
- `crates/bijux-policies/tests/surface/path_policies.rs`

## Why
Separating orchestration from effects keeps the engine deterministic, testable,
and free of platform-specific behavior.
