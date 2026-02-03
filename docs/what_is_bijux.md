# What Bijux Is

Bijux is a deterministic, contract-first workflow system for sequencing data.
It is built to make pipeline behavior explicit, repeatable, and testable across
machines and time.

## Philosophy

- **Contracts over conventions**: every stage and tool declares inputs, outputs,
  and validation rules. Execution must satisfy these contracts.
- **Determinism as a feature**: results are keyed by tool version, image digest,
  inputs, and parameters so the same inputs produce the same outputs.
- **Small, composable stages**: FASTQ processing is broken into clear stages
  (validate, trim, filter, merge, etc.) that can be composed and tested.

## Guarantees

- **Declared outputs**: stages are expected to produce only documented outputs.
- **Traceable runs**: manifests record tool, version, image digest, inputs, and
  parameters for every execution.
- **Stable UX**: command-line help and examples are snapshotted to prevent drift.

## What makes it different

- Bijux treats biological workflows like software systems: with interfaces,
  versioned contracts, and regression tests.
- It separates **domain definitions** (stages/tools/metrics) from **execution
  engines**, so a FASTQ domain can evolve independently from runtime details.
- It favors **observable artifacts** (metrics, manifests, logs) over implicit
  behavior.
