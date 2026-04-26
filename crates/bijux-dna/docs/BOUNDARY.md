# bijux-dna Boundary Contract

Owner: CLI
Scope: User-facing command adapter over API, registry, and domain-support commands
Allowed inputs: CLI arguments, current working directory, repository configs, API responses
Forbidden dependencies: engine internals, runner internals, direct product execution ownership
Forbidden effects: undeclared writes, network access, process spawning, hidden runtime mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --no-default-features`

## Why this crate exists
Owns the process-free and binary CLI entrypoints for planning, dry-run, status, registry, and
operator command behavior.

## Allowed dependencies
- API, config, policy, domain-compiler, runtime support, and CLI-facing helper crates required by
  the public command surface.
- No reverse-layer coupling into engine or runner internals.

## Allowed effects
- Controlled reads from repository configs and domain metadata.
- Controlled writes only to declared command outputs.
- No process spawning or network access from CLI adapters.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
