# Corpus validation tests

Run from the workspace root with build products under `artifacts/`:

```sh
CARGO_TARGET_DIR=artifacts/target cargo test -p bijux-dna-bench-corpus --locked
CARGO_TARGET_DIR=artifacts/target cargo test -p bijux-dna-bench-corpus --locked --no-default-features --features alignment
CARGO_TARGET_DIR=artifacts/target cargo test -p bijux-dna-bench-corpus --locked --no-default-features --features variants
CARGO_TARGET_DIR=artifacts/target cargo test -p bijux-dna-bench-corpus --locked --all-features
```

The contracts cover manifest rejection, paired reads, taxonomy and abundance
observations, damage fixtures, BAM sample/header and cohort checks, and VCF
reference/sample/variant admission. BAM and VCF cases compile with their
respective features. Fixtures resolve the repository at runtime and allocate
scratch products through the test harness.

Dependency regressions reject CLI, benchmark aggregation, execution and runtime
edges, mandatory native dependencies, and implicitly enabled native defaults.
These tests establish declared local admission behavior. They do not prove
interchangeability of every external scientific tool or performance on HPC.

## BAM assignment contracts

Owner regressions reuse the existing governed fixture/configuration corpus to test
valid admission, foreign sample rejection, non-finite posterior rejection, reference
build substitution and pairwise relationship loss. Small local files test canonical
path identity, including Unix aliases. Injected source failures verify acquisition
ordering and preserve previous output. Duplicate compatibility stages cannot overwrite.

Three original CLI contracts and the original inline routing assertions retain text,
JSON and TSV coverage. Default, alignment and all-feature owner inventories remain
separate; assignment regressions require alignment, and alias coverage requires Unix.
