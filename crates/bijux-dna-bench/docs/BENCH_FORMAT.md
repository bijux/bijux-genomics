# Benchmark Format

The stable persisted artifact set is:

- `observations.jsonl`
- `summary.json`
- `decision.json`
- `decisions.json`

Artifact writers canonicalize JSON before writing stable files. Field names and
ordering are part of the review surface because fixtures and snapshots compare
serialized output.

## observations.jsonl

`observations.jsonl` contains one canonical JSON object per line. Each line is a
`BenchmarkObservation`.

Required meaning:

- identifies dataset, stage, optional stage instance, optional lineage, tool, and
  parameter hash
- records runtime, memory, exit status, platform, runner, replicate identity, and
  metric envelope
- validates before it is accepted for summarization

## summary.json

`summary.json` is a `BenchmarkSummary`.

Required meaning:

- one deterministic summary row per comparable dataset/stage/tool/parameter
  group
- robust runtime, memory, and metric summaries
- low-power, completeness, failure-rate, and stratum evidence
- stable row ordering

## decision.json

`decision.json` is the compatibility single-decision artifact. It is written when
at least one gate decision exists and contains the first deterministic
`GateDecision`.

Required meaning:

- selected row identity
- pass/fail result
- rationale and violation evidence
- metric values used by the gate

## decisions.json

`decisions.json` is the complete deterministic list of `GateDecision` values for
the suite run.

Required meaning:

- every summary row evaluated by the policy is represented
- ordering follows summary-row ordering
- each decision validates before persistence

## Adding Fields

When adding artifact fields, update:

- `bijux-dna-bench-model`
- this file
- `docs/BENCH_CONTRACT.md`
- fixtures under `tests/fixtures/`
- contract or determinism tests that lock the new shape
