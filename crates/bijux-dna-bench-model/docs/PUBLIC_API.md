# bijux-dna-bench-model Public API

The crate exposes a curated namespace in `public_api` for benchmark model contracts:

- `compare` for deterministic summary comparison and typed comparison reports
- `contract` for schema IDs and validation entrypoints
- `BenchError` for stable validation and diagnostics failures
- benchmark model types such as `BenchmarkObservation`, `BenchmarkSuiteSpec`, `BenchmarkSummary`, and `BenchmarkDecision`
- policy types such as `GatePolicy`, `GateDecision`, and `GateViolation`
- statistics helpers such as `robust_stats`
