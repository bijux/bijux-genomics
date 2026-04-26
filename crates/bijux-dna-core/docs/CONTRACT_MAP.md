# Contract Map

This file is the SSOT for core contract families and their owning modules.

## Contract Families

| Family | Owning Module | Notes |
| --- | --- | --- |
| Canonical JSON | `src/contract/canonical.rs`, `src/foundation/canonical.rs` | Public canonical bytes live under `contract`; crate-local normalization helpers live under `foundation`. |
| Contract version | `src/contract/version.rs`, `src/foundation/version.rs` | Version records used by serialized contracts and foundation payloads. |
| Execution graph | `src/contract/execution/graph.rs` | `ExecutionGraph`, `ExecutionStep`, `ExecutionEdge`, graph validation, and deterministic graph hash. |
| Execution contract | `src/contract/execution/contract.rs` | Execution output validation and artifact contracts. |
| Execution manifest | `src/contract/execution/manifest.rs` | Execution manifest record shape. |
| Execution policy | `src/contract/execution/policy.rs` | `PlanPolicy` and retry/resource policy records. |
| Execution records | `src/contract/execution/record.rs` | Stage execution records and status contracts. |
| Stage IO | `src/contract/execution/io.rs` | Artifact specs, roles, paths, and IO cardinality. |
| Run metadata | `src/contract/run/metadata.rs` | Run, stage, tool execution, and invocation metadata records. |
| Run provenance | `src/contract/run/provenance.rs` | Scientific and tool provenance payloads. |
| Run index | `src/contract/run/index.rs` | Run index lines, query records, and run listing helpers. |
| Run spec/domain | `src/contract/run/spec.rs`, `src/contract/run/domain.rs` | Pipeline spec, domain records, and run request shape. |
| Tooling registry | `src/contract/tooling/mod.rs` | Stage specs, tool manifests, tool registry, tool invocation, and objective records. |
| Stage selection | `src/contract/tooling/selection/mod.rs` | Pure selection scoring and disqualification records. |
| Identifier catalog | `src/id_catalog/{pipeline,stage,tool}/` | Canonical pipeline, stage, and tool id constants. |
| Typed identifiers | `src/ids/typed/` | Pipeline, stage, tool, artifact, run, and step id wrappers. |
| Identifier parsing | `src/ids/parsing/` | Family-specific id parsing and symbolic validation. |
| Domain model | `src/ids/domain_model.rs` | Shared assay, domain, platform, layout, and library model records. |
| Metrics | `src/metrics/` | Metric ids, derived metric parsing, schema ids, registry constants, and metric payloads. |
| Input assessment | `src/foundation/input_assessment.rs` | FASTQ discovery, assessment records, hashing, and assessment persistence helper. |
| Managed operations | `docs/COMMANDS.md` | SSOT for callable operations exposed by core. |

## SSOT Rules

- Do not define new shared id types outside `src/ids/`.
- Do not define new canonical pipeline, stage, or tool constants outside
  `src/id_catalog/`.
- Do not re-implement canonical JSON or hashing rules outside this crate.
- Do not add serialized contract fields without following `docs/CHANGE_RULES.md`.
- Do not add callable core operations without updating `docs/COMMANDS.md`.
- Do not move input discovery or run-index query semantics into downstream
  crates; downstream crates may orchestrate calls, but the contract shape stays
  here.

## Public Access

Downstream crates should import through `bijux_dna_core::prelude` for ergonomic
use, or through narrower public modules when a focused dependency is clearer:
`contract`, `id_catalog`, `ids`, `metrics`, or `public_api`.
