# BENCH_FORMAT

## Stability statement
The JSON shapes and field ordering in `decision.json`, `observations.jsonl`, and `summary.json`
are versioned and stable. Breaking changes require a major bump and updated fixtures.

## decision.json
Fields:
- tool_id
- score
- rationale

Example:
```json
{
  "tool_id": "fastp",
  "score": 0.91,
  "rationale": ["lower adapter contamination", "higher retention"]
}
```

Invariants:
- tool_id must be canonical.
- score is deterministic.

## observations.jsonl
Fields:
- metric_id
- value
- units

Example (single line):
```json
{"metric_id":"retention_reads","value":0.92,"units":"ratio"}
```

Invariants:
- one observation per metric per stage.

## summary.json
Fields:
- aggregate scores
- best tool
- decision rationale

Example:
```json
{
  "best_tool": "fastp",
  "aggregate_score": 0.88,
  "rationale": ["higher retention", "lower contamination"]
}
```

## Adding a new benchmark dimension
- Add the new field to `docs/BENCH_CONTRACT.md` and update fixtures under `tests/fixtures/*`.
- Update any summary aggregation logic and include a deterministic ordering rule.
- Add or update a contract test under `tests/contracts/*` to lock the new shape.
