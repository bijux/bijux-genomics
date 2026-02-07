# Methodological Intent (FASTQ)

## What
Defines the scientific meaning of FASTQ stages.

## Why
Separates scientific intent from operational contracts.

## Non-goals
- Tool execution details.

## Contracts
Enforced by stage contracts and planner snapshots.

## Examples
- validate: ensures input is syntactically correct.
- trim: removes adapters/low-quality tails.
- merge: combines overlapping paired reads.
- screen: identifies contaminant signatures.

## Failure modes
If intent is unclear, update this file before changing stages.
