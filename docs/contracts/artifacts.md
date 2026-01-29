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

Pipeline-level summaries (when present) contain:

```
execution_manifest.json
```

## Definitions

- `run_manifest.json`: run-level artifact index (paths + checksums) for metrics, retention report,
  and the adapter bank reference used for the run.
- `manifest.json`: execution manifest for the tool invocation (tool/version/image/inputs/command).
- `metrics.json`: metrics envelope file (execution + stage metrics payload).
- `retention_report.json`: retention report placeholder in v1 schema.
- `adapter bank ref`: a `run_manifest.json` entry pointing at `assets/adapters/bank.v1.yaml` with a
  checksum.

## Requirements

- Filenames are stable and versioned.
- Artifacts are written only under the run directory.
- Missing or invalid artifacts are a hard failure.
