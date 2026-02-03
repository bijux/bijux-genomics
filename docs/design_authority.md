# Bijux Design Authority

Bijux is a scientific execution authority for bioinformatics workflows. It exists to make results comparable, auditable, and reproducible across tools and time.

## Guarantees

- **Determinism by design**: identical inputs + identical tool images + identical parameters produce identical outputs or explicit failures.
- **Explicit contracts**: every tool declares required inputs, expected outputs, and metrics; violations fail the run.
- **Traceable provenance**: each run records tool, image, parameters, environment, and metrics.
- **Schema governance**: metric schemas are versioned; mismatches are rejected or migrated.

## Refusals

- No silent fallbacks that change results.
- No implicit tool selection without recording rationale.
- No execution that bypasses validation gates.

## How Bijux Differs From Workflow Engines

Workflow engines orchestrate tasks; Bijux governs correctness. Bijux models domains, contracts, metrics, and invariants as first-class citizens, then executes only what it can prove is valid for that domain and input set.

## Scientific Interpretation

- Metrics are part of the result, not optional telemetry.
- Comparisons are explicit and reproducible; Bijux records what changed and why.
- Failures are data, and are recorded with provenance.

## Philosophy

Bijux is intentionally conservative: correctness and comparability over convenience. When a tradeoff exists, Bijux prefers explicitness and auditability.
