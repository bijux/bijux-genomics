# Refusals

## What
Explicit statements of what Bijux Genomics does not support.

## Why
Refusals prevent accidental scope creep and keep policies enforceable.

## Non-goals
- Supporting ad‑hoc shell execution in stages.
- Accepting untyped identifiers in contracts.

## Contracts
- Effect boundary policy.

## Examples
- Stages never build command lines.

## Failure modes
- Attempts to bypass contracts are rejected by policy tests.

## Stage Refusals
- `fastq.infer_asvs`
  - reason_code: `FASTQ_ASV_OUT_OF_SCOPE`
  - status: out_of_scope
  - issue: `FASTQ-3B-66`
  - policy: not admitted until deterministic/containerized ASV execution is certified.
