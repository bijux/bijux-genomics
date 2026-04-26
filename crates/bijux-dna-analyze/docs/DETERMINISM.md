# Determinism

## Contract
For identical inputs, feature flags, schema versions, and dependency versions,
`bijux-dna-analyze` must produce stable analysis outputs.

## Stable Outputs
- Loaded facts are sorted by run, stage, tool, params hash, and input hash.
- Report section names and table order are deterministic.
- `report.json`, `report_bundle/index.html`, and snapshot-normalized report outputs are stable.
- Ranking output uses explicit tie-breaks rather than incidental collection order.
- SQLite query tests must return stable latest-record selection when the `sqlite` feature is
  enabled.

## Allowed Variance
- Values read from input artifacts may vary when the upstream run varied.
- Wall-clock timestamps and durations may vary when they are upstream input data.
- Domain snapshot hashes are expected to change when governed domain inputs or docs change.

## Forbidden Variance
- Hash-map iteration order must not leak into reports.
- Filesystem traversal order must be sorted before it affects output.
- Optional feature support must fail explicitly when disabled; for example, parquet paths return
  `UnsupportedParquet` without the `parquet` feature.

## Coverage
- `tests/determinism/fixture_stability.rs`
- `tests/contracts/report/report_determinism.rs`
- `tests/contracts/pipeline/stable_ordering.rs`
- `tests/schemas/sqlite/sqlite_determinism.rs`
