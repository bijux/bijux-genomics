# bijux-dna

## What this crate does
`bijux-dna` owns the user-facing CLI process for the genomics workspace. It parses operator
commands, prepares process context, renders CLI output, and delegates durable genomics behavior to
stable API and infrastructure surfaces.

## Boundaries
This crate must not own domain science, planner policy, runner backends, stage execution, or hidden
side effects. Direct domain, stage, engine, runner, or runtime ownership is technical debt here; the
CLI boundary is documented in [docs/BOUNDARY.md](docs/BOUNDARY.md).

## Public entrypoints
- `src/bin/bijux-dna.rs` is the binary wrapper.
- `src/cli_entrypoint.rs` owns process-free CLI startup used by tests.
- `src/process_exit.rs` owns categorized operator exit behavior.
- `src/public_api/` is the curated testable library surface described in
  [docs/PUBLIC_API.md](docs/PUBLIC_API.md).

## Contracts and operating rules
The CLI owns command names, help stability, output rendering, and operator error shape. It consumes
API contracts from `bijux-dna-api` and infrastructure helpers for declared filesystem effects only.
The authoritative command inventory is [docs/COMMANDS.md](docs/COMMANDS.md).

## Effects And Determinism
Allowed effects are explicit reads of repository configuration plus declared command outputs. Dry-run
behavior is covered in [docs/DRY_RUN.md](docs/DRY_RUN.md), output formats in
[docs/OUTPUT_FORMATS.md](docs/OUTPUT_FORMATS.md), and effect rules in
[docs/EFFECTS.md](docs/EFFECTS.md).

## Tests
Use `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --no-default-features`.
Layout and dependency checks live in `tests/boundaries.rs`, command behavior in
`tests/contracts.rs`, dry-run coverage in `tests/contracts/dry_run.rs`, and snapshots in
`tests/snapshots/`.

## Docs
This crate has one root README and exactly ten crate docs under `docs/`. Start with
[docs/INDEX.md](docs/INDEX.md), then use [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the
source/test tree and [docs/TESTS.md](docs/TESTS.md) for verification mapping.

Repository work is governed by `/Users/bijan/bijux/bijux-genomics/README.md`,
`README.md`, and `README.md`; read those
files before changing this crate or making commits.
