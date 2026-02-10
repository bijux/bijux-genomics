# Architecture Litmus

## What
A quick checklist to validate architecture adherence.

## Why
Keeps design consistent across contributions.

## Non-goals
- Detailed implementation guide.

## Contracts
- Boundary map, SSOT, contract spine.
- engine does not depend on runner or environment
- prelude is exports-only
- defaults live only in bijux-dna-pipelines
- composition roots are only in API/CLI
- Domain is authored SSOT; configs are generated; code consumes generated configs; makefiles call CLI only.

## Examples
- Engine depends only on Runner trait, not concrete runner.

## Failure modes
- Policy tests fail if dependencies violate boundaries.
