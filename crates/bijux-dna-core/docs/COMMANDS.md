# Commands

This file is the SSOT for callable operations owned by `bijux-dna-core`.

## Managed Core Operations

These operations are pure Rust entrypoints or typed contract helpers. This crate
does not own CLI command parsing, process execution, workflow orchestration, or
report rendering.

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `canonicalize-json` | `contract::canonicalize_json_value` | Sort and normalize JSON contract values deterministically. |
| `canonical-json-bytes` | `contract::to_canonical_json_bytes` | Serialize contract values to stable canonical JSON bytes. |
| `params-hash` | `prelude::params_hash` | Hash normalized parameter JSON. |
| `parameters-fingerprint` | `prelude::parameters_fingerprint` | Build deterministic parameter fingerprints. |
| `input-fingerprint` | `prelude::input_fingerprint` | Build deterministic input fingerprints from input hashes. |
| `discover-fastq-files` | `prelude::input_assessment::discover_fastq_files` | Discover FASTQ paths in stable order. |
| `assess-input-dir` | `prelude::input_assessment::assess_input_dir` | Build a typed FASTQ input assessment from a directory. |
| `write-input-assessment` | `prelude::input_assessment::write_input_assessment` | Persist an input assessment payload. |
| `validate-execution-graph` | `contract::ExecutionGraph::new` | Validate execution graph shape and deterministic contract invariants. |
| `validate-execution-outputs` | `contract::validate_execution_outputs` | Validate declared execution outputs against the execution contract. |
| `select-stage` | `contract::select_stage` | Select a stage from typed tooling candidates without running tools. |
| `query-run-index` | `contract::{list_runs, query_runs, query_run, query_latest_runs, query_stage_rows}` | Query typed run index records. |
| `parse-pipeline-id` | `ids::PipelineId` and `ids::parsing` | Validate and construct pipeline identifiers. |
| `parse-stage-id` | `ids::StageId` and `ids::parsing` | Validate and construct stage identifiers. |
| `parse-tool-id` | `ids::ToolId` and `ids::parsing` | Validate and construct tool identifiers. |
| `validate-metric-id` | `metrics::validate_metric_id_str` | Validate metric identifier syntax. |
| `metrics-schema-for-stage` | `metrics::metrics_schema_for_stage` | Resolve the metrics schema registered for a stage. |

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-core --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test semantics --no-default-features
```
