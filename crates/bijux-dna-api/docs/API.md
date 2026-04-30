# API

The stable public API is `bijux_dna_api::v1::api`. The crate root exports only
`pub mod v1`; the front door decides which v1 operations and helper namespaces
are public.

## Stable Operations

| Operation | Request | Response | Notes |
| --- | --- | --- | --- |
| `plan` | `PlanRequest` | `PlanResponse` | Returns the execution graph, graph hash, manifest preview, workflow manifest, plan manifest, and optional semantic diff. |
| `execute` | `ExecuteRequest` | `ExecuteResponse` | Runs a graph with a declared runtime and output directory. |
| `execute_and_report` | `ExecuteRequest` | `ExecuteResponse` | Executes and materializes report outputs through the run/reporting adapter. |
| `dry_run` | `DryRunRequest` | `DryRunResponse` | Writes deterministic graph and manifest artifacts without executing stages. |
| `status` | run identifier/path input | `RunStatus` | Reads persisted manifest/report status for a run. |
| `pause_run` | run directory path input | `RunControlResponse` | Persists a pause request in the governed run-control record. |
| `resume_run` | run directory path input | `RunControlResponse` | Persists a resume request in the governed run-control record. |
| `cancel_run` | run directory path input | `RunControlResponse` | Persists a cancellation request in the governed run-control record. |
| `operator_health` | run directory path input | `OperatorHealthResponse` | Writes and returns the governed operator-health report for a run root. |
| `browse_runs` | `RunBrowserRequestV1` | `RunBrowserResponseV1` | Builds a typed run-browser index with filtering/pagination and optional redaction profile. |
| `query_run_lineage` | `RunLineageQueryRequestV1` | `RunLineageQueryResponseV1` | Returns stable lineage edges from artifact inventory input-lineage records. |
| `cache_explain` | `CacheExplainRequestV1` | `CacheExplainResponseV1` | Returns typed cache key fingerprints and miss reasons. |
| `replay_explain` | `ReplayExplainRequestV1` | `ReplayExplainResponseV1` | Returns typed replay reuse/rerun/change summaries. |
| `evidence_gap` | `EvidenceGapRequestV1` | `EvidenceGapResponseV1` | Returns typed evidence gaps, failed checks, and trust-class caveats. |
| `operator_diagnosis` | `OperatorDiagnosisRequestV1` | `OperatorDiagnosisResponseV1` | Returns diagnosis commands bound to run-state/queue/control/health/failure contracts. |
| `sign_bundle_prototype` | `SignedBundleRequestV1` | `SignedBundleResponseV1` | Writes `bundle_signature.json` using the prototype signing contract. |
| `verify_signed_bundle_prototype` | `SignedBundleVerifyRequestV1` | `SignedBundleVerifyResponseV1` | Verifies bundle signature against current governed bundle hashes. |
| `render_run_browser_output` | `(RunBrowserResponseV1, OutputFormatV1)` | `String` | Produces stable human or canonical JSON output. |
| `render_operator_diagnosis_output` | `(OperatorDiagnosisResponseV1, OutputFormatV1)` | `String` | Produces stable human or canonical JSON output. |
| `explain` | execution graph plus optional defaults ledger | `ExplainResponse` | Returns selected tools, defaults diff, and stage contract evidence. |
| `policy_audit` | audit target input | policy audit JSON | Reports the policy-audit owner and commands without executing policy guardrails from runtime API code. |
| `render_report` | `RenderReportRequest` | `RenderReportResult` | Renders a report bundle for existing run facts. |

`docs/COMMANDS.md` is the SSOT for the full command list and local verification
commands.

## Planner Manifest Surfaces

`PlanRequest` may now carry an optional governed `workflow_manifest`, emitted
`stage_plans`, explicit parameter traces, stable planner warnings/refusals, and
an optional `compare_against` baseline manifest.

`PlanResponse` now returns:

- `workflow_manifest`: the caller-supplied workflow manifest or the API's
  deterministic fallback synthesis from the execution graph.
- `plan_manifest`: the governed planner contract with ordered steps, cache keys,
  stage decisions, parameter traces, and cross-domain handoff checks.
- `plan_diff`: an optional semantic diff when the request supplied
  `compare_against`.

## Schema Contracts

Stable schema-bearing types include:

- `PlanRequest`
- `PlanResponse`
- `ExecuteRequest`
- `ExecuteResponse`
- `DryRunRequest`
- `DryRunResponse`
- `RunStatus`
- `RunBrowserRequestV1`
- `RunBrowserResponseV1`
- `RunLineageQueryRequestV1`
- `RunLineageQueryResponseV1`
- `CacheExplainRequestV1`
- `CacheExplainResponseV1`
- `ReplayExplainRequestV1`
- `ReplayExplainResponseV1`
- `EvidenceGapRequestV1`
- `EvidenceGapResponseV1`
- `OperatorDiagnosisRequestV1`
- `OperatorDiagnosisResponseV1`
- `OutputFormatV1`
- `RedactionProfileV1`
- `SignedBundleRequestV1`
- `SignedBundleResponseV1`
- `SignedBundleVerifyRequestV1`
- `SignedBundleVerifyResponseV1`
- `RunControlResponse`
- `OperatorHealthResponse`
- `RenderReportRequest`
- `RenderReportResult`
- `ExplainResponse`
- `ExplainToolSelection`
- `PlanExplainV1`

The schema snapshot tests under `tests/schemas/` and `tests/snapshots/` are the
review boundary for public shape changes.

## Helper Namespaces

The front door exposes curated helper namespaces for callers that need direct v1
access:

- `api::plan`
- `api::run`
- `api::report`
- `api::bench`
- `api::bam`
- `api::fastq`
- `api::env`
- `api::shared`

These namespaces must remain curated. Do not re-export lower-level crates
wholesale.

## Stability Rules

- Adding optional fields with stable defaults is usually compatible.
- Removing fields, changing field meaning, renaming operations, or changing
  serialized shapes is breaking.
- Breaking changes require explicit approval, updated snapshots, and updates to
  this file, `docs/PUBLIC_API.md`, and `docs/CHANGE_RULES.md`.
- Internal modules under `src/internal/`, `src/runtime/`, `src/support/`, and
  `src/surface/` are not public entrypoints unless re-exported by `v1::api`.
