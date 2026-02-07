# BENCH_FORMAT

## decision.json
Fields:
- tool_id
- score
- rationale

Invariants:
- tool_id must be canonical
- score is deterministic

## observations.jsonl
Fields:
- metric_id
- value
- units

Invariants:
- one observation per metric per stage

## summary.json
Fields:
- aggregate scores
- best tool
- decision rationale
