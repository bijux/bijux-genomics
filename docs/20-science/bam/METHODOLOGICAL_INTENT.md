# Methodological Intent (BAM)

## What
Defines the scientific meaning of BAM stages.

## Why
Separates intent from operational contract.

## Non-goals
- Tool execution details.

## Contracts
- Stage-level purpose and inputs/outputs live in [STAGE_CATALOG.md](STAGE_CATALOG.md).
- Stage-level interpretation limits live in [STAGE_ASSUMPTIONS.md](STAGE_ASSUMPTIONS.md).
- The execution/report boundary lives in [OPERATIONAL_CONTRACT.md](OPERATIONAL_CONTRACT.md).

## Examples
- align: map reads to reference.
- markdup: identify duplicates.
- damage: estimate deamination patterns.

## Failure modes
If intent is unclear, update this file before changing stages.
