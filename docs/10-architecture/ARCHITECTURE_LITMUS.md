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

## Examples
- Engine depends only on Runner trait, not concrete runner.

## Failure modes
- Policy tests fail if dependencies violate boundaries.
