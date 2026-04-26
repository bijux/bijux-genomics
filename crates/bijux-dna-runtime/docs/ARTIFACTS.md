# Runtime Artifacts

Runtime writes only declared runtime artifacts under run-layout roots and tool-run directories.

## Run layout root
From `RunLayout`:
- `RunLayoutV1` contract snapshot: `tests/fixtures/runtime_schema/default/run_layout_v1.json`
- `execution_manifest.json` (engine emits execution plan manifest)
- `RunManifest` contract snapshot: `tests/fixtures/runtime_schema/default/run_manifest_v1.json`
- `RunRecordV1` contract snapshot: `tests/fixtures/runtime_schema/default/run_record_v1.json`
- `RunProvenanceV1` contract snapshot: `tests/fixtures/runtime_schema/default/run_provenance_v1.json`
- `run_metadata.json`
- `environment.json`
- `events.jsonl`
- `stages/` (per-stage outputs)
- `summary/` (reporting outputs)

## Tool-run artifacts
Under `tools/<tool>/run/<run_id>/`:
- `manifest.json`
- `metrics.json`
- `run_manifest.json`
- `profile_manifest.json`
- `run_manifest.lock.json`
- `artifacts/`
- `logs/`
- `run_artifacts/`

## Run-artifact subdirectories
Under `<tool-run>/run_artifacts/`:
- `telemetry/events.jsonl`
- `telemetry/timings.json`
- `telemetry/resources.json`
- `telemetry/errors.json`
- `dashboard/facts.jsonl`
- `reproducibility/report.json`
- `plans/*.json`

## Recorder Rules
- The recorder is the only write path for runtime manifests, records, metrics, telemetry, provenance, and support files.
- Canonical JSON writes must go through `write_canonical_json` or a more specific runtime writer.
- No ad hoc JSON serialization should bypass recording helpers for owned artifacts.
- `logs/` may receive bounded stdout/stderr tails, but process execution remains outside runtime.

## Links to schema fixtures
- `tests/fixtures/runtime_schema/default/run_layout_v1.json`
- `tests/fixtures/runtime_schema/default/run_manifest_v1.json`
- `tests/fixtures/runtime_schema/default/run_record_v1.json`
- `tests/fixtures/runtime_schema/default/run_provenance_v1.json`
