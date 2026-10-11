# Dependencies

`bijux-dna-db-ena` is a leaf data-source adapter. It may depend on external
libraries needed for CLI parsing, HTTP, JSON, parallel transfer, and error
handling, but it must not depend on downstream workspace layers.

## Normal Dependencies

- `anyhow`: binary and transfer error context.
- `clap`: helper-binary argument parsing.
- `md5`: ENA source checksum verification during governed transfer.
- `rayon`: bounded parallel download execution.
- `reqwest`: blocking HTTP client for ENA metadata and file endpoints.
- `serde`: typed contract serialization.
- `serde_json`: manifest persistence.
- `sha2`: local content identity derivation for admitted downloads.
- `tempfile`: private no-clobber transfer publication.
- `thiserror`: typed client and query validation errors.

## Dev Dependencies

- `bijux-dna-policies`: crate-local guardrail smoke coverage.

## Forbidden Workspace Dependencies

This crate must not depend on `bijux-dna`, API, planner, engine, runner,
runtime, stage, environment, domain, benchmark, or reference-database crates.
Those crates may consume ENA records or tasks; they must not be pulled into this
adapter.

## Verification

The dependency boundary is locked by `tests/boundaries/dependency_graph.rs`.
