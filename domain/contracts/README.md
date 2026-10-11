# Artifact contracts

This directory publishes the canonical artifact guarantees shared by workflow planners,
stage handoffs, and benchmark admission. Contracts describe stable scientific properties such
as format, compression, cardinality, pairing, sort and index state, identities, sidecars,
lineage, and validation invariants. Tool output filenames are not part of compatibility.

The public Rust authority is `bijux-dna-stage-contract`, backed by low-level model primitives in
`bijux-dna-core`. Regenerate or verify the checked files with:

```console
cargo run -p bijux-dna-dev -- tooling run artifact-contracts
cargo run -p bijux-dna-dev -- tooling run artifact-contracts -- --check
```

A producer is compatible with a consumer only when every possible producer format, compression,
pairing mode, and sort order is accepted by the consumer, and every required consumer identity,
index, sidecar, lineage guarantee, and validation invariant is guaranteed by the producer.
