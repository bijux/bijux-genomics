# Bijux API Policy

This crate is the public, stable entrypoint for orchestration. The API is curated and versioned.

Rules:
1. All public surface is versioned (e.g. `bijux_api::v1`).
2. Avoid re-exporting internal crates wholesale. Only expose explicit types/functions required
   for external consumers.
3. Power-user/internal exports must be behind the `api_internal` feature.
4. Any change that grows the public surface requires updating the public surface snapshot test.
5. The API surface should remain compatible within a version namespace (v1, v2, ...).

Modules:
v1 surface (curated modules only):
- v1::plan: pipeline selection + plan building.
- v1::run: execution entrypoints + runtime helpers.
- v1::report: report rendering and report helpers.
- v1::bench: comparison/benchmark helpers + domain constants for benchmarking.

Anything outside `v1` is internal and not part of the public contract.
