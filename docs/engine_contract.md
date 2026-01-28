# Engine Contract

This document defines the hard boundary for the Bijux engine.

## Engine CAN

- run commands
- track resources
- enforce invariants
- emit manifests

## Engine CANNOT

- know FASTQ (or any domain-specific semantics)
- know stages beyond generic input/output contracts
- compute deltas
- rank tools
- decide policies

## Boundary Tests

The engine must not import any domain crates. Tests enforce this in
`crates/bijux-engine/tests/architecture.rs`.
