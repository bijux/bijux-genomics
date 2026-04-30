# Runtime Contract

`bijux-dna-runtime` defines the stable runtime contract for run layouts, manifests, records, provenance, observability contracts, telemetry events, and runner handoff types.

## Terms
- **Run**: one end-to-end pipeline execution.
- **Layout**: deterministic directory structure for a run.
- **Record**: per-step or per-run execution record.
- **Manifest**: stable runtime summary contract containing graph hash, tool identity, artifacts, and layout metadata.
- **Provenance**: immutable metadata describing tool version, image digest, parameters, and inputs.
- **Telemetry**: schema-stable event stream and summary records emitted during runtime.

## Owned JSON artifacts
- `RunLayoutV1`
- `RunManifest`
- `RunRecordV1`
- `RunProvenanceV1`

## Executor Governance Surfaces
Runtime exposes governed contracts for execution backends and operator controls so local, container, and HPC behavior can be inspected and validated consistently.

Core surfaces:
- Smoke workflow plans and validation for Docker/Apptainer runners.
- Slurm submission records, lifecycle transitions, and mocked submission builders.
- Site-isolated HPC profile selection (`lunarc`) decoupled from cluster-local secrets or account facts.
- Executor capability negotiation and semantic-safe fallback decisions.
- Runtime resource admission and scheduling decision assembly.
- Queue restore decisions with duplicate-dispatch prevention.
- Idempotent pause/resume/cancel control actions with append-only audit records.
- Run-layout storage isolation checks for traversal and out-of-root paths.

See `ARTIFACTS.md` for the full file inventory under a run layout.

## Telemetry Contract
Telemetry uses `bijux.telemetry.v1` JSONL events.

Stable fields:
- `schema_version`
- `run_id`
- `stage_id`
- `tool_id`
- `event_name`
- `status`
- `trace_id`
- `span_id`
- `attrs`

Intentionally unstable fields:
- `timestamp`
- `duration_ms`

Event names include `run_started`, `stage_start`, `tool_invocation`, `stdout_summary`, `stderr_summary`, `metrics_emitted`, `invariant_result`, `artifact_written`, `stage_end`, `run_finished`, `run_failed`, `merge_decision`, `adapter_validation`, `contaminant_action`, `quality_gate`, and `error`.

Failure codes include `tool_failed`, `missing_artifact`, `invalid_params`, `invariant_violation`, `io_error`, `timeout`, `parse_error`, and `unknown`.

Sensitive telemetry attributes containing terms such as `token`, `secret`, `password`, `api_key`, or `authorization` are redacted.

## Compatibility
Runtime schema changes are versioned. Older runs remain loadable by keeping parsers backward compatible or by introducing explicit versioned contracts.

Additive fields can be backward compatible when defaults are documented and tests cover older payloads. Removing fields, renaming fields, changing enum encodings, changing canonical hashes, or changing fixture schemas is breaking unless explicitly approved.

## Reference Walkthrough
1. Create a run layout.
2. Prepare a tool-run directory under `tools/<tool>/run/<run_id>/`.
3. Write runtime support files, execution manifest, tool-run manifest, profile manifest, and lock manifest.
4. Downstream consumers read emitted manifests and run artifacts.

Key artifacts in the reference story:
- `execution_manifest.json`
- `tools/<tool>/run/<run_id>/run_manifest.json`
- `tools/<tool>/run/<run_id>/run_artifacts/telemetry/events.jsonl`

## Reference
See `tests/contracts/reference/reference_example.rs` for a minimal end-to-end example.

## Example
```json
{ "schema_version": "bijux.run_manifest.v1", "graph_hash": "sha256:..." }
```

## Change Rules
- Contract changes must update docs and tests together.
- Schema changes must update fixture snapshots in the same change.
- Runtime writer changes must preserve canonical JSON where required.
- Telemetry taxonomy changes must update telemetry contract and golden tests.
