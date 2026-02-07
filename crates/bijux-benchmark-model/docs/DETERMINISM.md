# DETERMINISM

## Randomness
Bootstrap sampling is the only randomness.

## Seeding rules
- Seed must be provided and recorded.
- Same seed => same outputs.

## Enforcement
Tests assert stable results with fixed seeds.

## Examples
Deterministic scoring:
`score_suite(suite, observations, seed)`

Deterministic gating:
`classify_gate(summary, policy, seed)`
