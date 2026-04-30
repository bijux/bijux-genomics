# Run Operations

Governed run operations for `feat/deep-foundation` execution backends live under the run root.

Core evidence files:
- `backend_descriptor.json`
- `scheduling_decision.json`
- `queue_state.json`
- `run_lease.json`
- `run_control.json`
- `operator_health.json`
- `slurm_submission.json` when the run is scheduled as `slurm_batch_candidate`
- `run_failure.json`, `run_state.json`, `evidence_bundle.json`, and `evidence_verification.json`

## Operator Checklist

Before touching a run:
- confirm the run root from `run_manifest.json`
- inspect `run_state.json` and `queue_state.json`
- confirm whether `run_lease.json` is still held
- read `operator_health.json` before retrying or resuming
- preserve `run_failure.json` and `evidence_bundle.json`; do not delete evidence during triage

## Common Failures

### Lease conflict
- Diagnosis command:
  `cat run_lease.json`
- Evidence path:
  `run_lease.json`
- Remediation:
  confirm the holder process is gone, then rerun through the governed API so a new lease is written cleanly
- Safety caveat:
  do not remove `run.lock` while another worker may still be writing artifacts

### Queue stalled in paused state
- Diagnosis command:
  `cat run_control.json`
- Evidence path:
  `run_control.json`
- Remediation:
  issue a governed resume request or rewrite the control file through `bijux_dna_api::v1::api::resume_run`
- Safety caveat:
  do not edit `queue_state.json` by hand; only the control surface should move a run out of pause

### Cancelled run with partial artifacts
- Diagnosis command:
  `cat run_failure.json`
- Evidence path:
  `run_failure.json`
- Remediation:
  inspect the active step from `queue_state.json`, decide whether the partial outputs are disposable, then relaunch through a fresh run directory if cleanup is uncertain
- Safety caveat:
  cancellation is auditable; preserve `run_control.json` and `run_failure.json` for replay and postmortem work

### Container runtime unavailable
- Diagnosis command:
  `cat operator_health.json`
- Evidence path:
  `operator_health.json`
- Remediation:
  restore `docker`, `apptainer`, or `singularity` on `PATH`, then rerun the health check before execute
- Safety caveat:
  do not switch backends silently after planning; regenerate the run so the backend descriptor and scheduler decision stay truthful

### Slurm candidate not submitted
- Diagnosis command:
  `cat scheduling_decision.json`
- Evidence path:
  `scheduling_decision.json` and `slurm_submission.json`
- Remediation:
  confirm the run classified as `slurm_batch_candidate`, then materialize or inspect the submission record before trying site-specific launch steps
- Safety caveat:
  the current branch records governed Slurm submission state for monitoring and tests; site integration still needs cluster-owned execution wrappers

## Recovery Flow

1. Read `operator_health.json`.
2. Read `run_state.json`, `queue_state.json`, and `run_control.json`.
3. If the run failed, preserve `run_failure.json` and verify `evidence_bundle.json`.
4. If the lease is stale, clear it only after proving the holder is inactive.
5. Resume, cancel, or relaunch through the governed API surface instead of editing runtime state files directly.

## Verification

- Runtime contract lane:
  `cargo test -p bijux-dna-runtime --test schemas runtime_operations_schema_snapshots -- --nocapture`
- API contract lane:
  `cargo test -p bijux-dna-api --test contracts v1_dry_run_manifest -- --nocapture`
- Execute gate:
  `cargo run -q -p bijux-dna-dev -- tooling run cargo-targets essential-execute`
