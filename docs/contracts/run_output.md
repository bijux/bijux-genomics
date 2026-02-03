# Bijux run output contract (v1)

Status: **frozen**. Changes require a version bump.

Every tool execution (QA or benchmark) emits a deterministic run directory:

```
run/
  manifest.json
  metrics.json
  artifacts/
  logs/
```

## Definitions

- `manifest.json`: execution manifest with tool, version, image digest, command, input hashes, and environment details.
- `metrics.json`: structured metrics payload (execution + stage metrics).
- `artifacts/`: tool outputs (FASTQ, reports, etc).
- `logs/`: deterministic logs from the tool/container (one log file per run).

## Determinism

- Paths and filenames are stable.
- Artifacts are written only under `artifacts/`.
- Logs are written only under `logs/`.
- Runs are keyed by a deterministic `run_id` derived from stage, tool, image digest, input hash, and params hash.

## Guarantees

- Inputs are mounted read-only; tools cannot mutate source FASTQs.
- Outputs must match the tool execution contract.
- Any deviation is a hard failure.
