# Commands

This file is the SSOT for callable operations owned by `bijux-dna-core`.

## Managed Core Operations

These operations are pure Rust entrypoints or typed contract helpers. This crate
does not own CLI command parsing, process execution, workflow orchestration, or
report rendering.

### Canonical Serialization And Identity

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `canonicalize-json` | `contract::canonical::canonicalize_json_value` | Sort and normalize JSON contract values deterministically. |
| `canonicalize-parameters-json` | `contract::canonical::parameters_json_canonicalization` | Normalize parameter JSON before hashing. |
| `canonicalize-truth-json` | `contract::canonical::canonicalize_truth_json` | Normalize manifest, record, and report JSON before serialization. |
| `canonical-json-bytes` | `contract::canonical::to_canonical_json_bytes` | Serialize contract values to stable canonical JSON bytes. |
| `params-hash` | `prelude::params_hash` | Hash normalized parameter JSON. |
| `parameters-fingerprint` | `prelude::parameters_fingerprint` | Build deterministic parameter fingerprints. |
| `input-fingerprint` | `prelude::input_fingerprint` | Build deterministic input fingerprints from input hashes. |
| `run-id-from-hashes` | `prelude::hashing::run_id_from_hashes` | Derive deterministic run identity from pipeline, sample, parameter, input, and optional reference hashes. |

### Identifier Validation

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `parse-pipeline-id` | `ids::parse_pipeline_id` | Validate and construct pipeline identifiers. |
| `validate-pipeline-id` | `ids::validate_pipeline_id_str` | Validate pipeline identifier strings. |
| `parse-stage-id` | `ids::parse_stage_id` | Validate and construct stage identifiers. |
| `validate-stage-id` | `ids::validate_stage_id_str` | Validate stage identifier strings. |
| `parse-tool-id` | `ids::parse_tool_id` | Validate and construct tool identifiers. |
| `validate-tool-id` | `ids::validate_tool_id_str` | Validate tool identifier strings. |
| `validate-artifact-id` | `ids::validate_artifact_id_str` | Validate artifact identifier strings. |
| `validate-profile-id` | `ids::validate_profile_id_str` | Validate profile identifier strings. |

### FASTQ Input Assessment

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `discover-fastq-files` | `prelude::input_assessment::discover_fastq_files` | Discover FASTQ paths in stable order. |
| `detect-fastq-path` | `prelude::input_assessment::is_fastq_path` | Classify FASTQ and gzipped FASTQ paths by extension. |
| `detect-gzip-path` | `prelude::input_assessment::is_gzip_path` | Classify gzip-compressed paths by extension. |
| `assess-input-dir` | `prelude::input_assessment::assess_input_dir` | Build a typed FASTQ input assessment from a directory. |
| `write-input-assessment` | `prelude::input_assessment::write_input_assessment` | Persist an input assessment payload. |

### Execution, Runs, And Tooling

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `validate-execution-graph` | `contract::ExecutionGraph::new` | Validate execution graph shape and deterministic contract invariants. |
| `hash-execution-graph` | `contract::ExecutionGraph::hash` | Hash an execution graph through canonical JSON bytes. |
| `normalize-execution-graph` | `contract::ExecutionGraph::normalize` | Sort graph steps and edges while revalidating graph shape. |
| `topological-step-ids` | `contract::ExecutionGraph::topological_step_ids` | Return deterministic topological step order for an acyclic graph. |
| `validate-execution-outputs` | `contract::validate_execution_outputs` | Validate declared execution outputs against the execution contract. |
| `query-run-index` | `contract::{list_runs, query_runs, query_run, query_latest_runs, query_stage_rows}` | Query typed run index records. |
| `build-run-dir` | `contract::run_dir` | Derive the canonical run directory path under a caller-owned base directory. |
| `select-stage` | `contract::select_stage` | Select a stage from typed tooling candidates without running tools. |
| `objective-spec` | `contract::objective_spec` | Resolve objective weights for pure stage selection. |

### Metrics

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `parse-metric-id` | `metrics::parse_metric_id` | Parse known metric identifiers. |
| `parse-derived-metric-id` | `metrics::parse_derived_metric_id` | Parse known derived metric identifiers. |
| `validate-metric-id` | `metrics::validate_metric_id_str` | Validate metric identifier syntax. |
| `validate-derived-metric-id` | `metrics::validate_derived_metric_id_str` | Validate derived metric identifier syntax. |
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
