# ARCHITECTURE

Public surface: `src/v1/*`.
Internal wiring: `handlers/`, `internal/`, and crate-private modules.

`v1/api.rs` is the front door for plan/execute/dry-run/explain/status.
Handlers implement the concrete wiring for FASTQ/BAM/cross pipelines.
