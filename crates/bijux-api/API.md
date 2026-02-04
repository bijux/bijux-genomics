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
- v1::plan: pipeline selection + plan building.
- v1::run: execution entrypoints.
- v1::report: report rendering and report helpers.
- v1::bench: comparison/benchmark helpers.
- v1::env: runtime + image resolution helpers.
- v1::fastq/bam: domain-specific helpers needed by CLI.

Anything outside `v1` is internal and not part of the public contract.
