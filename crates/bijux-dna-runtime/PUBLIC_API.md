# bijux-dna-runtime Public API

Public modules exported from `src/lib.rs`:
- `environment`
- `manifests`
- `observability`
- `provenance`
- `recording`
- `run`
- `run_layout`
- `runner`
- `telemetry`

Root re-exports are intentionally limited to the stable entrypoints used across crates:
- Observability contracts: `RunProvenanceV1`, `RunContextV1`, `TelemetryEventV1`, report types, and telemetry validation helpers.
- Recording entrypoints: `prepare_tool_run_dirs`, `write_canonical_json`, `write_profile_and_lock_manifests`, `write_run_manifest`.
- Run layout entrypoints: `create_run_layout`, `write_manifest`, `RunManifest`, `RunStageEntry`.
- Runner entrypoints: `ensure_stage_supported_by_runner`, `Artifact`, `Invocation`, `Runner`, `RunnerContractKind`, `RunnerResult`.
- Telemetry adapter entrypoints: `build_telemetry_adapter`, `TelemetryAdapter`, `TelemetrySpan`.

Items that are not listed above should be consumed from their owning namespace instead of the root facade.
