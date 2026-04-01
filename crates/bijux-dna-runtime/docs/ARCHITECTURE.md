# Architecture

The canonical runtime contract lives in `RUNTIME_CONTRACT.md`. This file maps the crate tree to clear ownership boundaries so the implementation stays navigable over time.

## Module map
- `run/`: profile loading, run-id generation, and run-base path resolution.
- `run_layout/`: run layout contracts, layout writers, and run-journal persistence.
- `recording/`: runtime file emitters for manifests, metrics, envelopes, provenance, and telemetry.
- `recording/manifests/`: tool-run directories, runtime support files, artifact catalogs, and manifest assembly.
- `manifests/`: runtime registry loading, source resolution, and stage classification helpers.
- `observability/`: schema-only report and telemetry contracts.
- `provenance/`: scientific provenance assembly from resolved tool invocations.
- `runner/`: runner execution models and stage-runner contracts.
- `telemetry/`: adapter selection plus persisted run-journal event models.

## Pointers
- `RUNTIME_CONTRACT.md` for the authoritative contract narrative.
- `ARTIFACTS.md` for run-layout and tool-run file inventory.
- `TESTS.md` for stability and integrity coverage.
