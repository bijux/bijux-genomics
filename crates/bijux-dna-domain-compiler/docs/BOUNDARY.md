# bijux-dna-domain-compiler Boundary Contract

Owner: Domain compiler
Scope: Compile authored domain source into generated config views and validation diagnostics
Allowed inputs: domain source files, compiler options, governed config destinations
Forbidden dependencies: runner, CLI, engine internals, runtime execution backends
Forbidden effects: product execution, network access, undeclared writes outside governed outputs
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-compiler --no-default-features`

## Why this crate exists
Validates domain source and emits generated registry/stage/config views consumed by the rest of the
workspace.

## Allowed dependencies
- Domain crates and infra helpers required to parse, validate, and render generated config views.
- No product execution or runtime backend ownership.

## Allowed effects
- Read authored domain source.
- Write only declared generated config outputs when invoked for generation.

## Notes
The repository policy is rooted at `README.md`, and the repository coding
policy is rooted at `README.md`.
