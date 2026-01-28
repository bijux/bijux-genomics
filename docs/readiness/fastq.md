# FASTQ Domain Readiness Checklist

## Invariants
- Core vs optional stage invariants are defined and versioned
- Mutating vs observational stages are explicit
- Delta metrics are required for trim/filter

## Metrics
- One metric schema per stage (fastq_*_v1)
- Execution metrics use bijux-measure ExecutionMetrics
- Domain metrics serialize via MetricEnvelope

## Tests
- Stage boundary compatibility tests
- Determinism regression tests
- Tool contract tests
- Observability schema tests

## Artifacts
- Canonical run layout enforced
- input_assessment.json present and immutable
- run_metadata.json present and complete
- events.jsonl present for every run

## Release Gate
- All above tests green
- Docs updated (fastq.md + fastq_runs.md)
- CLI help snapshots updated
