# FASTQ Contributor Contract

This file is a guardrail. If you add or change FASTQ tools, you must follow it.

## Invariants (must never be broken)
- Reads are never duplicated silently.
- Pairing is preserved unless the stage contract allows it to break.
- Output is normalized to canonical names at stage boundaries.
- Metrics must pass schema validation.

## Stage Contracts
- Every tool must obey the `FastqStageContract` for its stage.
- If a tool violates the contract (even with exit code 0), the run fails.

## Deltas
- All stage transitions emit a `FastqDelta`.
- Do not compute deltas in tool-specific code. Use `bijux_domain_fastq::domain`.

## Adding a Tool
1) Add the tool manifest in `domain/fastq/tools/`.
2) Ensure the tool outputs match stage outputs exactly.
3) Confirm `bijux-domain-fastq` can normalize outputs for the stage.
4) Run tests: `make test lint security`.

## What Not To Do
- Do not add FASTQ semantics outside `bijux_domain_fastq::domain`.
- Do not skip header inspection or compatibility gates.
- Do not add ad-hoc metrics or delta logic.
