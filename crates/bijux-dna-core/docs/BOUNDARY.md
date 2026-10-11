# bijux-dna-core Boundary Contract

Owner: low-level contract primitives, identifiers, metrics, canonicalization,
hashing, and foundation model helpers. `bijux-dna-stage-contract` owns the
public stage and artifact contract product surface.

## Owns

- Public contract types for execution graphs, canonical artifact compatibility,
  execution records, run records, run metadata, provenance, tooling, and stage
  selection.
- Canonical JSON rules and canonical byte serialization.
- Hashing helpers for parameters, inputs, files, and reproducibility identity.
- Typed identifiers and parsing/validation for pipeline, stage, tool, artifact,
  run, step, and profile ids.
- Canonical identifier catalogs for pipelines, stages, and tools.
- Shared metric identifiers, derived metrics, schemas, registry lookup, and
  metric envelopes.
- FASTQ input assessment contract records and deterministic directory
  assessment helpers.
- Stable public API mirrors and prelude ergonomics.

## Does Not Own

- CLI command parsing or command-line UX.
- Workflow planning, stage implementation, tool execution, scheduling, or
  runtime orchestration.
- Product API handlers or report rendering.
- Environment provisioning, container runtime behavior, or backend selection.
- Domain-specific metric interpretation beyond shared core metric contracts.
- Context-specific identifiers and validation for benchmark attempts, scheduler jobs,
  qualification receipts, repository objects, and product-owned protocol records.

## Allowed Effects

Most core operations are pure in-memory transformations. The explicit exception
is `foundation::input_assessment`, which reads input files to discover FASTQ
paths and hash file contents, and can persist a typed assessment payload when a
caller invokes `write_input_assessment`.

This crate must not spawn processes, perform network access, call product APIs,
or mutate runtime state.

## Forbidden Dependencies

`bijux-dna-core` must not depend on downstream orchestration or product layers:

- `bijux-dna`
- `bijux-dna-api`
- `bijux-dna-analyze`
- `bijux-dna-bench`
- `bijux-dna-engine`
- `bijux-dna-containers`
- `bijux-dna-pipelines`
- planner crates
- runner/runtime crates
- domain or stage crates

Normal dependencies must remain generic library dependencies. Dev-only test
dependencies may use policy/test crates to run repository guardrails, but normal
dependencies must not include any `bijux-*` workspace crate. Any dependency that
imports pipeline policy, command execution, or product behavior into core is a
boundary violation.

`tests/boundaries/dependency_graph.rs` locks the current normal dependency set
and rejects workspace crate dependencies in `[dependencies]`.

## Validation

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-core --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test boundaries --no-default-features
```
