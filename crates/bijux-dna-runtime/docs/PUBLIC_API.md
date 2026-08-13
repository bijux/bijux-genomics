# Public API

`bijux-dna-runtime` exposes stable runtime contracts, recording helpers, run-layout helpers, runner handoff traits, and telemetry adapters. Consumers should prefer root re-exports for stable entrypoints and use module namespaces for owner-specific details.

## Public Modules
- `attempt_sequence`
- `environment`
- `manifests`
- `observability`
- `provenance`
- `recording`
- `run`
- `run_layout`
- `runner`
- `telemetry`

## Root Exports
- Attempt sequence entrypoints: `reserve_attempt_sequence`, `supersede_attempt_sequence`, `AttemptSequenceIdentity`, `AttemptSequenceReservation`, `ATTEMPT_SEQUENCE_RESERVATION_SCHEMA_V1`.
- Observability contracts: `RunProvenanceV1`, `RunContextV1`, `TelemetryEventV1`, report types, telemetry event names, failure codes, attribute redaction, and telemetry validation helpers.
- Recording entrypoints: `initialize_attempt_lifecycle`, `ensure_attempt_lifecycle`, `append_attempt_lifecycle_transition`, `load_attempt_lifecycle`, `load_attempt_scheduler_observations`, `benchmark_attempt_directory`, `record_attempt_ready`, `record_attempt_submission`, `record_attempt_running`, `record_attempt_scheduler_observation`, `record_attempt_accounting`, `record_attempt_acceptance`, `record_attempt_blocked_external`, `record_attempt_superseded`, typed attempt records, `prepare_tool_run_dirs`, `write_canonical_json`, `write_profile_and_lock_manifests`, `write_run_manifest`.
- Run layout entrypoints: `create_run_layout`, `write_manifest`, `write_run_state`, `write_runtime_policy`, `write_executor_descriptor`, `write_checkpoint`, `write_failure_record`, `RunManifest`, `RunStageEntry`.
- Runner entrypoints: `ensure_stage_supported_by_runner`, `Artifact`, `Invocation`, `Runner`, `RunnerContractKind`, `RunnerResult`.
- Telemetry adapter entrypoints: `build_telemetry_adapter`, `TelemetryAdapter`, `TelemetrySpan`.

## Stability Rules
- Additive root exports require this document and the public API test to change together.
- Non-additive changes to owned JSON contracts require schema fixture updates.
- Use `run_layout::*` module surfaces for governed run-state, runtime-policy, executor-descriptor, checkpoint, failure, and deterministic layout contracts.
- Items not listed above should be consumed from their owning module namespace.

## Source Authorities
- `src/lib.rs` controls public module visibility and root re-exports.
- `stable_surface.rs` files curate module-level stable exports.
- `docs/RUNTIME_CONTRACT.md` defines behavioral stability.

## Stability Tiers

- Stable: the root exports and public modules listed above when consumed through the documented crate-root paths.
- Experimental: no experimental runtime namespace is exported today; new opt-in surfaces must be listed here before downstream use.
- Internal: helper modules, private support code, and any item not listed under Root Exports or Public Modules.
