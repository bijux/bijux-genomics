# Architecture Litmus

## What
A quick checklist to validate architecture adherence.

## Why
Keeps design consistent across contributions.

## Non-goals
- Detailed implementation guide.

## Contracts
- Dependency-layer litmus rules live in [BOUNDARY_MAP.md](BOUNDARY_MAP.md).
- Contract ownership litmus lives in [CONTRACT_SPINE.md](CONTRACT_SPINE.md).
- Single-source-of-truth litmus lives in [SSOT.md](SSOT.md).
- Crate-family responsibility and placement litmus lives in [CRATE_AUTHORITY_MAP.md](CRATE_AUTHORITY_MAP.md).
- engine does not depend on runner or environment
- prelude is exports-only
- defaults live only in bijux-dna-pipelines
- composition roots are only in API/CLI
- Domain is authored SSOT; configs are generated; code consumes generated configs; makes call CLI only.

## Examples
- Engine depends only on Runner trait, not concrete runner.

## Failure modes
- Policy tests fail if dependencies violate boundaries.
