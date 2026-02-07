# Runtime Artifacts

## Run layout root
From `RunLayout`:
- `run_layout.json` (schema: `tests/fixtures/runtime_schema/run_layout_v1.json`)
- `execution_manifest.json` (engine emits execution plan manifest)
- `run_manifest.json` (schema: `tests/fixtures/runtime_schema/run_manifest_v1.json`)
- `run_record.json` (schema: `tests/fixtures/runtime_schema/run_record_v1.json`)
- `run_provenance.json` (schema: `tests/fixtures/runtime_schema/run_provenance_v1.json`)
- `run_metadata.json`
- `environment.json`
- `events.jsonl`
- `stages/` (per-stage outputs)
- `summary/` (reporting outputs)

## Per-stage run artifacts
Under `stages/<stage_id>/run_artifacts/`:
- `effective_config.json`
- `tool_invocation.json`
- `metrics.json`
- `stage_report.json`
- `execution_record.json`
- `metrics_envelope.json`
- `stage_metrics.json`
- `invocations/<stage_id>.tool_invocation.json`

## Links to schema fixtures
- `tests/fixtures/runtime_schema/run_layout_v1.json`
- `tests/fixtures/runtime_schema/run_manifest_v1.json`
- `tests/fixtures/runtime_schema/run_record_v1.json`
- `tests/fixtures/runtime_schema/run_provenance_v1.json`
