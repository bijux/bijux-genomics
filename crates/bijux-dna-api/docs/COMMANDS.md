# Commands

This file is the SSOT for commands and callable operations owned by
`bijux-dna-api`.

## Managed API commands

These v1 operations are exported through `bijux_dna_api::v1::api`.

| Command | Rust entrypoint | Purpose |
| --- | --- | --- |
| `plan` | `plan(PlanRequest)` | Build a stable execution graph plus governed workflow/plan manifest surfaces without running stages. |
| `execute` | `execute(ExecuteRequest)` | Run a planned request in `simulation`, `advisory`, or `enforced` mode and return governed run-state, policy, checkpoint, and failure pointers. |
| `execute-and-report` | `execute_and_report(ExecuteRequest)` | Run execution and materialize report outputs through one API call. |
| `dry-run` | `dry_run(DryRunRequest)` | Validate inputs and emit deterministic dry-run graph, run manifest, plan manifest, run-state, runtime-policy, executor-descriptor, and checkpoint artifacts. |
| `status` | `status(run_id)` | Read persisted run state and return `RunStatus` with governed runtime contract pointers when present. |
| `pause-run` | `pause_run(run_dir)` | Persist a governed pause request and append an auditable control transition for the selected run. |
| `resume-run` | `resume_run(run_dir)` | Persist a governed resume request so a paused run can continue at the next safe checkpoint. |
| `cancel-run` | `cancel_run(run_dir)` | Persist a governed cancellation request and let execution terminate through the control-aware runner boundary. |
| `operator-health` | `operator_health(run_dir)` | Recompute and persist the operator health report for storage, container runtime, queue, executor, and evidence linkage checks. |
| `explain` | `explain(plan, defaults_ledger)` | Build the explainability bundle for a planned graph. |
| `policy-audit` | `policy_audit(...)` | Return the policy-audit owner and commands; policy execution stays in `bijux-dna-dev` and `bijux-dna-policies`. |
| `render-report` | `render_report(RenderReportRequest)` | Render an existing run report bundle. |
| `render-report-html` | `render_report_bundle_html(...)` | Render the v1 HTML report bundle. |
| `workspace-edges` | `workspace_edges(...)` | Inspect workspace dependency edges for audit/reporting flows. |
| `write-workspace-audit` | `write_workspace_audit(...)` | Persist workspace audit evidence for API-managed runs. |

## Namespace helpers

The public front door also exposes curated v1 namespaces:

- `api::bench` for benchmark helper exports.
- `api::plan` for planning helper exports.
- `api::run` for run orchestration helper exports.
- `api::report` for report helper exports.
- `api::bam` for BAM helper exports.
- `api::fastq` for FASTQ helper exports.
- `api::env` for environment helper exports.
- `api::shared` for shared v1 helper exports.

## Local verification commands

Run these commands from the `bijux-genomics` repository root.

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-api --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test contracts --no-default-features
```

Use `--all-features` for release-facing validation when changes touch feature-gated
reporting, benchmark, Docker-runner, or internal API behavior.
