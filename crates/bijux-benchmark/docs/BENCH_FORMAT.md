# BENCH_FORMAT

## Stability statement
The JSON shapes and field ordering in `decision.json`, `observations.jsonl`, and `summary.json` are versioned and stable. Breaking changes require a major bump and updated fixtures.

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

## Adding a new benchmark dimension
- Add the new field to `docs/BENCH_CONTRACT.md` and update fixtures under `tests/fixtures/*`.
- Update any summary aggregation logic and include a deterministic ordering rule.
- Add or update a contract test under `tests/contracts/*` to lock the new shape.
