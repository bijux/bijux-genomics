# Architecture

The canonical runtime contract lives in `RUNTIME_CONTRACT.md`. This file maps the crate tree to ownership boundaries so the implementation stays navigable and contract changes stay deliberate.

## Module map
- `lib.rs` exports public modules and a curated stable root facade.
- `environment.rs` owns environment fingerprint contracts.
- `run/` owns profile loading, run-id generation, run-base path resolution, and stable run exports.
- `run_layout/` owns run-layout contracts, layout creation, run-layout writers, run-journal persistence, and stable run-layout exports.
- `recording/` owns runtime file emitters for manifests, metrics, envelopes, provenance, telemetry, canonical JSON, checksums, logs, and stable recording exports.
- `recording/manifests/` owns tool-run directories, runtime support files, artifact catalogs, manifest identity, profile/lock manifests, reproducibility reports, run-manifest assembly, and stable manifest writer exports.
- `manifests/` owns governed registry loading, source resolution, generated registry conversion, domain registry conversion, stage classification helpers, and stable manifest registry exports.
- `observability/` owns schema-only report contracts and telemetry validation contracts.
- `observability/reports/` owns run and stage report schemas.
- `observability/telemetry/` owns telemetry attrs, event taxonomy, facts rows, and provenance schema contracts.
- `provenance/` owns parameter and scientific provenance assembly from resolved tool invocations.
- `runner/` owns runner execution models, runner traits, stage-support checks, runner contract kinds, and stable runner exports.
- `telemetry/` owns telemetry adapter selection plus persisted run-journal event models.

## Source Rules
- Keep process execution outside runtime; runtime defines runner contracts but does not call backend commands.
- Keep filesystem writes in `run_layout/` and `recording/`, through canonical or governed writers.
- Keep observability schemas separate from artifact writing.
- Keep stable module exports in `stable_surface.rs` files.
- Add new files only for durable ownership concerns, not as temporary staging areas.

## Pointers
- `RUNTIME_CONTRACT.md` for the authoritative contract narrative.
- `ARTIFACTS.md` for run-layout and tool-run file inventory.
- `BOUNDARY.md`, `DEPENDENCIES.md`, and `EFFECTS.md` for architectural limits.
- `TESTS.md` for stability, integrity, privacy, and layout coverage.
