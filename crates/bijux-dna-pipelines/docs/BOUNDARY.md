# bijux-dna-pipelines Boundary Contract

Owner: Pipelines
Scope: deterministic pipeline profile composition, profile manifests, defaults ledgers, and registry lookup contracts.

## Allowed Inputs
- Domain contracts from FASTQ, BAM, and VCF domain crates.
- Core typed identifiers and deterministic hashing helpers.
- Policy and runtime model crates only in tests that validate repository guardrails and downstream contract shape.

## Forbidden Dependencies
- Runner backends, engine execution, CLI adapters, planner crates, stage implementations, databases, environment discovery, and science package orchestration.
- Any dependency that makes this crate own execution, scheduling, external tool discovery, or command invocation.

## Forbidden Effects
- Process spawning.
- Network access.
- Product execution.
- Undeclared file writes outside test fixtures and snapshot-controlled artifacts.

## Allowed Effects
- Pure deterministic profile construction.
- Deterministic serialization for manifests, ledgers, hashes, and snapshots.
- Fixture-backed tests that validate boundary, registry, and defaults contracts.

## Validation
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-pipelines --no-default-features
```
