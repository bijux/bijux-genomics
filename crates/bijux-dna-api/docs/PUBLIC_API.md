# Public API

This crate has one stable public namespace:

```rust
pub mod v1;
```

Everything else in `src/lib.rs` is crate-private wiring. Public callers should
enter through `bijux_dna_api::v1::api`.

## Front Door Exports

`src/v1/api/front_door.rs` exports:

- Operations: `plan`, `execute`, `execute_and_report`, `dry_run`, `status`,
  `explain`, `policy_audit`, `render_report`, `render_report_bundle_html`,
  `workspace_edges`, and `write_workspace_audit`.
- Contract types: `PlanRequest`, `PlanResponse`, `ExecuteRequest`,
  `ExecuteResponse`, `DryRunRequest`, `DryRunResponse`, `RunStatus`,
  `RenderReportRequest`, `RenderReportResult`, `ExplainResponse`,
  `ExplainToolSelection`, `PlanExplainV1`, and `VcfRunRequest`.
- Curated helper namespaces: `bench`, `plan`, `run`, `report`, `bam`, `fastq`,
  `env`, and `shared`.

## Export Policy

- Export only stable, versioned APIs from `v1`.
- Prefer narrow re-exports over exposing complete lower-level crates.
- Keep internal handler, runtime, support, and surface modules crate-private.
- Add new public operations to `docs/COMMANDS.md` in the same change.
- Add or update schema snapshots for any public shape change.

## Non-Public Surfaces

The following modules are intentionally private implementation detail:

- `src/internal/`
- `src/runtime/`
- `src/support/`
- `src/surface/`

They may change when the v1 public contract remains stable.

## Stability Tiers

- Stable: `pub mod v1`, `bijux_dna_api::v1::api`, and the contract types/operations documented above.
- Experimental: no experimental public namespace is exported today; any future one must be called out explicitly here before use.
- Internal: `src/internal/`, `src/runtime/`, `src/support/`, `src/surface/`, and any item not re-exported through `v1::api`.
