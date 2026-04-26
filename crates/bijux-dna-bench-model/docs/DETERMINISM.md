# Determinism

`bijux-dna-bench-model` must return the same output for the same typed input.
This is a model crate, so hidden clocks, global randomness, filesystem state,
network state, and process state are outside its boundary.

## Randomness

`stats::bootstrap_ci(values, samples, seed)` is the only crate-owned operation
that samples. The seed is part of the function input and must be supplied by the
caller. `stats::seed_from_ids` can derive a stable seed from suite, metric,
stage, and tool ids when a caller needs repeatable bootstrap intervals.

Forbidden source patterns include unseeded `fastrand`, `rand::random`,
`thread_rng`, and entropy-seeded RNG construction. The determinism tests scan
source files for those patterns.

## Stable Ordering

Outputs must be deterministic when maps, sets, graph edges, metrics, or
rationale traces are involved:

- Use `BTreeMap`, `BTreeSet`, explicit sorting, or stable input order.
- Sort floating-point values with a total fallback for incomparable values.
- Keep `missing_metrics`, `violations`, comparison diffs, and outlier indices
  stable for identical inputs.
- Do not use hash map iteration order as public output order.

## Examples

Deterministic scoring example:

```text
score_suite(suite, observations, seed)
```

Deterministic gating example:

```text
classify_gate(summary, policy, seed)
```

The example names above document the intended caller-level workflow. The crate's
current pure entrypoints are listed in `docs/COMMANDS.md`.

## Verification

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
```
