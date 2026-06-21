# Runtime Iteration 09 Operator Commands

This note captures the operator-facing command surface added for iteration 09 runtime proofs.

## Local end-to-end workflow execution

- `execute_local_fastq_workflow(<run_dir>)`
- `execute_local_bam_workflow(<run_dir>)`
- `execute_local_vcf_workflow(<run_dir>)`

All three execute in enforced local mode and emit governed runtime artifacts, evidence bundle files, replay manifests, and hash ledgers.

## Runtime identity and replay analysis

- `environment_identity(<run_dir>)`
- `explain_successful_replay(<original_run_dir>, <replay_run_dir>)`
- `assess_failed_replay_eligibility(<run_dir>)`
- `replay_failed_run(<run_dir>)`
- `explain_cache_hit_miss(<original_run_dir>, <replay_run_dir>)`

These APIs provide replay provenance and cache-miss explainability without mutating source-run evidence.

## Failure injection and run-bundle verification

- `run_local_failure_injection(<run_dir>, <scenario>)`
- `verify_run_bundle(<run_dir>)`

Supported `run_local_failure_injection` scenarios:

- `timeout`
- `cancel`
- `missing_output`
- `corrupt_output`
- `nonzero_exit`
- `interrupted_process`
- `partial_files`
