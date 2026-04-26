# bijux-dna-api Public API

Public modules exported from `src/lib.rs`:
- v1

Stable versioned entrypoints exposed under `src/v1/`:
- `v1::api`
- `v1::bench`
- `v1::plan`
- `v1::run`
  routed through `src/v1/run/`
  with `entrypoints.rs`, `request_contracts.rs`, `runtime_support.rs`, and `operator_failure.rs`
- `v1::report`
  routed through `src/v1/report/`
  with `request_contracts.rs`, `analysis_exports.rs`, and `html_bundle.rs`
- `v1::env`
- `v1::bam`
- `v1::fastq`
