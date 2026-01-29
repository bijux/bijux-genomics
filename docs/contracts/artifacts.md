# Bijux artifacts contract (v1)

Status: **draft**. Changes require a version bump.

This contract defines the exact filenames emitted by the FASTQ pipeline. These names are stable
and should be treated as API.

## Guaranteed filenames

Each tool run directory contains:

```
run_manifest.json
manifest.json
metrics.json
retention_report.json
artifacts/
logs/
```

Trim runs also emit:

```
run_artifacts/adapters/effective_adapters.json
run_artifacts/adapters/adapter_bank_ref.json
run_artifacts/reports/adapter_trimming_report.json
run_artifacts/reports/retention_report.json
```

Pipeline-level summaries (when present) contain:

```
execution_manifest.json
```

## Definitions

- `run_manifest.json`: run-level artifact index (paths + checksums) for metrics, retention report,
  and adapter artifacts used for the run.
- `manifest.json`: execution manifest for the tool invocation (tool/version/image/inputs/command).
- `metrics.json`: metrics envelope file (execution + stage metrics payload).
- `retention_report.json`: retention report placeholder in v1 schema.
- `effective_adapters.json`: resolved adapter set for the selected preset (ids + sequences).
- `adapter_bank_ref.json`: adapter bank reference with checksums, preset, and enabled ids.
- `adapter_trimming_report.json`: adapter trimming report (placeholder counts until tool parsing).
- `retention_report.json`: retention report (pre/post) snapshot for the stage.

## Requirements

- Filenames are stable and versioned.
- Artifacts are written only under the run directory.
- Missing or invalid artifacts are a hard failure.
