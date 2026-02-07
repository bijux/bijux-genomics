# SSOT

## What This Crate Owns
This crate is the single source of truth for:

- **ID newtypes**: `RunId`, `StepId`, `StageId`, `ToolId`, `ArtifactId`, `ProfileId`.
- **Contract versions** and compatibility rules.
- **Canonical serialization** rules and hashing.
- **Execution and run contracts**: graph, manifest, run record, tool invocation.

## What Is Forbidden Elsewhere
Other crates must not:

- Define new ID types or reuse raw `String` for public IDs.
- Serialize contract JSON with ad-hoc `serde_json::to_writer` or custom ordering.
- Re-implement hashing or path normalization.
- Invent new contract fields or alter shapes without versioning here.

## Allowed Extensions
Other crates may extend contracts by:

- Adding metrics types in domain crates.
- Adding stage contracts in stage-contract.

But any new public contract surface must be declared here first and versioned.
