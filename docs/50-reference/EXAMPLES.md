# EXAMPLES

Canonical examples index.

## Runnable Example Inventory
- [examples/index.yaml](../../examples/index.yaml)
- [examples/README.md](../../examples/README.md)

Runnable example IDs are generated in [examples/index.yaml](../../examples/index.yaml). This document must not duplicate
that list manually.

Canonical example contract files also live with the example directories and are discovered from
the `canonical_example: true` entries in [examples/index.yaml](../../examples/index.yaml):
- `tiny-inputs.json`
- `workflow-manifest.json`
- `expected-evidence.json`

## Non-Runnable Example Surfaces
- [examples/_template/README.md](../../examples/_template/README.md)
- [examples/data/corpus-01/README.md](../../examples/data/corpus-01/README.md)
- [examples/data/corpus-01-mini/README.md](../../examples/data/corpus-01-mini/README.md)

## Recipe-Only Benchmark Docs
- [examples/fastq/index-reference-bench/README.md](../../examples/fastq/index-reference-bench/README.md)
- [examples/fastq/normalize-abundance-bench/README.md](../../examples/fastq/normalize-abundance-bench/README.md)

## Root Example Guide
- [examples/README.md](../../examples/README.md)
- [examples/POLICY.md](../../examples/POLICY.md)
- [examples/RECIPE_ONLY.txt](../../examples/RECIPE_ONLY.txt)

## Purpose
Define the navigation contract for runnable examples, recipe-only benchmark docs, and example-linked corpora.

## Scope
Applies to the `examples/` tree, the generated example index, and the docs that explain how example classes differ.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- [examples/index.yaml](../../examples/index.yaml) is the SSOT for runnable example IDs only.
- Canonical example contract details belong to the example directories referenced by
  [examples/index.yaml](../../examples/index.yaml), not to a duplicated table in this file.
- [examples/_template/README.md](../../examples/_template/README.md) and
  [examples/data/corpus-01/README.md](../../examples/data/corpus-01/README.md) /
  [examples/data/corpus-01-mini/README.md](../../examples/data/corpus-01-mini/README.md) are
  navigable docs, not runnable example IDs.
- Recipe-only benchmark docs are intentionally excluded from `examples/index.yaml` until they grow an executable example contract.
- `examples/data/` holds corpora inputs and can appear in navigation docs without being treated as runnable examples.
