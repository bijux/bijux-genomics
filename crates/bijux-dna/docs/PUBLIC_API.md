# bijux-dna Public API

`bijux-dna` exposes a narrow public Rust surface for the binary and integration tests.

## Exports
- `run_from_env`: process entrypoint used by the binary wrapper.
- `run_from_args`: process-free entrypoint for tests and callers with explicit argv/cwd.
- `public_api::run_with_args`: command execution helper over parsed argv.
- `public_api::run_with_cli`: command execution helper over a parsed `Cli`.
- `public_api::cli`: CLI parser and argument types.
- `public_api::hpc`: HPC layout contract helpers used by CLI-facing tests.

## Internal-Only Surfaces
These are implementation details and must not become stable exports:

- `commands/router/`
- `commands/cli/`
- `commands/support/`
- `commands/planning/`
- `commands/status/`
- `commands/corpus/`
- `commands/benchmark/`
- `commands/example/`
- `commands/fastq/meta/`
- `commands/bam/`
- `commands/vcf/`
- `process_exit` internals beyond the public module contract

## Rules
- Add public exports only when tests or downstream crates need a stable CLI integration point.
- Do not export command internals as a shortcut around architecture boundaries.
- Keep domain/stage execution behind API-owned surfaces.
- Keep `lib.rs` exports synchronized with the public-surface snapshot.

## Verification
- `tests/schemas/public_surface.rs`
- `tests/snapshots/bijux-dna__schemas__public_surface.snap`
