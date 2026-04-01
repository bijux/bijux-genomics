# Architecture

The canonical runtime contract lives in `RUNTIME_CONTRACT.md`. This file maps the crate tree to clear ownership boundaries so the implementation stays navigable over time.

## Module map
- `run/`: profile loading, run-id generation, run-base path resolution, and a dedicated stable surface.
- `run_layout/`: run layout contracts, layout creation, layout writers, run-journal persistence, and a dedicated stable surface.
- `recording/`: runtime file emitters for manifests, metrics, envelopes, provenance, telemetry, and a dedicated stable surface.
- `recording/manifests/`: tool-run directories, runtime support files, artifact catalogs, manifest assembly, and a dedicated stable surface.
- `manifests/`: runtime registry loading, source resolution, stage classification helpers, and a dedicated stable surface.
- `observability/`: schema-only report and telemetry contracts.
- `provenance/`: scientific provenance assembly from resolved tool invocations.
- `runner/`: runner execution models, stage-runner contracts, runner-contract kinds, and a dedicated stable surface.
- `telemetry/`: adapter selection plus persisted run-journal event models.

## Pointers
- `RUNTIME_CONTRACT.md` for the authoritative contract narrative.
- `ARTIFACTS.md` for run-layout and tool-run file inventory.
- `TESTS.md` for stability and integrity coverage.
