# Public API

`bijux-dna-runner` exposes a small runner facade plus backend and step-runner modules needed by current workspace callers. Internal support modules remain private.

## Public Modules
- `backend`
- `command_runner`
- `public_api`
- `step_runner`

## Root Exports
- `api`
- `ApptainerRunner`
- `DockerRunner`
- `LocalRunner`

## Facade Exports
- `BackendKind`
- `build_tool_execution_spec`
- `parse_mem_to_mb`
- `replay_run`
- `invocation_hash`
- `run_command`
- `run_command_with_context`
- `ExecutionBindingManifestV1`
- `CommandExecutionBindingV1`
- `ExecutionBindingManifestV2`
- `CommandExecutionBindingV2`
- `BoundFileIdentityV1`
- `RUNNER_EXECUTION_BINDINGS_ENV`
- `RUNNER_EXECUTION_BINDINGS_SCHEMA_V1`
- `RUNNER_EXECUTION_BINDINGS_SCHEMA_V2`
- `CommandOutputV1`
- `execute_observer_command`
- `execute_external_artifact_step`
- `execute_step`
- `StageResultV1`
- `submit_slurm_job`
- `submit_slurm_batch`
- `prepare_benchmark_slurm_job`
- `submit_prepared_benchmark_job`
- `load_benchmark_slurm_job_handle`
- `prepare_managed_slurm_job`
- `submit_prepared_managed_slurm_job`
- `load_managed_slurm_job_handle`
- `load_managed_slurm_job_spec`
- `load_prepared_managed_slurm_job`
- `observe_managed_slurm_job`
- `collect_managed_slurm_job`
- `ManagedSlurmJobSpec`
- `ManagedSlurmJobResources`
- `PreparedManagedSlurmJob`
- `ManagedSlurmJobCollection`
- `ManagedSlurmJobEvidence`
- `observe_slurm_job`
- `collect_slurm_job`
- `cancel_slurm_job`
- `SlurmClientPaths`
- `SlurmSubmitRequest`
- `SlurmDependencyCondition`
- `SlurmArraySpec`
- `SlurmArrayTaskAccounting`
- `SlurmJobHandle`
- `SlurmBatchSubmission`
- `SlurmSubmissionFailure`
- `SlurmJobObservation`
- `SlurmAccounting`
- `SlurmJobState`
- `SlurmEvidenceSource`
- `SLURM_JOB_HANDLE_SCHEMA_V1`
- `PreparedBenchmarkSlurmJob`
- `prepare_corpus_materialization_slurm_job`
- `submit_prepared_corpus_materialization_job`
- `load_corpus_materialization_slurm_job_handle`
- `load_corpus_materialization_slurm_spec`
- `load_prepared_corpus_materialization_slurm_job`
- `observe_corpus_materialization_slurm_job`
- `collect_corpus_materialization_slurm_job`
- `CorpusMaterializationSlurmSpec`
- `CorpusMaterializationSlurmResources`
- `PreparedCorpusMaterializationSlurmJob`
- `CorpusMaterializationCollection`
- `CorpusMaterializationEvidence`
- `CorpusMaterializationEvidenceStatus`
- `CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1`
- `CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1`
- `CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1`
- `prepare_container_qualification_slurm_job`
- `submit_prepared_container_qualification_job`
- `load_container_qualification_slurm_job_handle`
- `load_container_qualification_slurm_spec`
- `load_prepared_container_qualification_slurm_job`
- `observe_container_qualification_slurm_job`
- `collect_container_qualification_slurm_job`
- `cancel_container_qualification_slurm_job`
- `plan_container_qualification_cycle`
- `advance_container_qualification_cycle`
- `status_container_qualification_cycle`
- `collect_container_qualification_cycle`
- `admit_container_qualification_cycle`
- `load_container_qualification_cycle_preparation`
- `archive_failed_container_qualification_attempt`
- `rollback_failed_container_qualification_attempt_archive`
- `ContainerQualificationPreparationRequest`
- `ContainerQualificationSlurmSpec`
- `ContainerQualificationSlurmResources`
- `PreparedContainerQualificationSlurmJob`
- `ContainerQualificationCollection`
- `ContainerQualificationAccountingRecord`
- `ContainerQualificationSchedulerEvidence`
- `ContainerQualificationSchedulerStatus`
- `ContainerQualificationCycleAction`
- `ContainerQualificationCyclePreparation`
- `ContainerQualificationCycleRequest`
- `ContainerQualificationCycleState`
- `ContainerQualificationCycleStatus`
- `ContainerQualificationCycleSummary`
- `ContainerQualificationCycleUnit`
- `CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3`
- `CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4`
- `CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5`
- `CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1`
- `CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1`
- `CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V1`
- `CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V1` (deserialize compatibility)
- `CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V2`
- `CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1`

## Stability Rules
- Prefer `bijux_dna_runner::api::*` for consumer-facing use.
- `ApptainerRunner`, `DockerRunner`, and `LocalRunner` are the concrete runtime adapters exported at the crate root for higher layers.
- Backend modules may expose backend contracts, but planning and CLI concerns must stay outside this crate.

## Source Authorities
- `src/lib.rs` controls public module visibility and root re-exports.
- `src/public_api/stable_surface.rs` curates facade exports.
- `src/backend/stable_surface.rs` and `src/backend/docker/stable_surface.rs` curate backend exports.

## Stability Tiers

- Stable: `api`, `ApptainerRunner`, `DockerRunner`, `LocalRunner`, and the documented facade exports above.
- Experimental: backend-specific additions are experimental until they are listed under Facade Exports or Root Exports in this file.
- Internal: support modules and any runner/backend helper not re-exported through the documented stable surfaces.

## Managed Slurm Workers

`ManagedSlurmJobSpec` and its prepare, submit, observe, and collect functions expose the generic
runner-owned lifecycle for an exact external worker. The API binds source, worker, resources,
dependencies, attempt layout, scheduler identity, accounting, and log hashes while leaving product
semantics to the worker's owning crate.
