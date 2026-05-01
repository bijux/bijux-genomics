# bijux-dna-api

`bijux-dna-api` is the stable, versioned Rust API for planning, dry-run,
execution, reporting, explainability, and policy-audit workflows in
`bijux-genomics`.

This crate follows repository governance documentation. `/Users/bijan/bijux/bijux-genomics/README.md`,
`README.md`, and `README.md`; re-read
those files before editing this child repository and before committing.

## What this crate does

This crate owns:

- The curated public `v1` API surface under `src/v1/api/front_door.rs`.
- Stable request and response contracts under `src/surface/`.
- Runtime adapters that coordinate planners, runners, reports, manifests, and
  policy audits under `src/runtime/`.
- API-local support for workspace resolution, reference resolution, tool
  selection, benchmark runtime selection, and QA gates under `src/support/`.
- Private handler wiring under `src/internal/`.

## Boundaries

This crate does not own domain algorithms, planner semantics, runner command
execution, environment discovery, or analyzer storage internals. Those contracts
belong to their dedicated crates and are consumed here through typed APIs.
Declared effects are limited to runtime/reporting/audit paths documented in
`docs/BOUNDARY.md`, `docs/REQUEST_FLOW.md`, `docs/SECURITY.md`, and the command
contracts.

## Public entrypoints

The SSOT for callable API operations is `docs/COMMANDS.md`. The main managed
commands are:

- `plan`
- `execute`
- `execute-and-report`
- `dry-run`
- `status`
- `explain`
- `policy-audit`
- `render-report`
- `render-report-html`
- `workspace-edges`
- `write-workspace-audit`

## Contracts and operating rules

- `src/lib.rs` exposes `pub mod v1` and keeps all other modules crate-private.
- `src/v1/` is the only stable public namespace.
- `src/surface/` holds schema and explainability contracts.
- `src/runtime/` adapts requests into runtime, reporting, validation, audit, and
  invocation-policy behavior.
- `src/support/` contains API-local helpers that are not stable public exports.
- `src/internal/` contains private cross-domain and FASTQ handler wiring.

See `docs/ARCHITECTURE.md` and `docs/BOUNDARY.md` for the full layout and
dependency direction.

## Documentation

The crate root intentionally has only this `README.md`. All other crate docs live
under `docs/`, with a 10-document allowance enforced by the boundary tests:

- `docs/API.md`
- `docs/ARCHITECTURE.md`
- `docs/BOUNDARY.md`
- `docs/CHANGE_RULES.md`
- `docs/COMMANDS.md`
- `docs/FEATURES.md`
- `docs/PUBLIC_API.md`
- `docs/REQUEST_FLOW.md`
- `docs/SECURITY.md`
- `docs/TESTS.md`

## Tests

Run crate checks from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-api --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test contracts --no-default-features
```

Primary test entrypoints are `tests/boundaries.rs`, `tests/schemas.rs`, and
`tests/contracts.rs`.
