# Change Rules

## Breaking Changes
Treat these as breaking unless a policy or product decision explicitly says otherwise:

- removing or renaming a command in [COMMANDS.md](COMMANDS.md)
- changing a required flag or positional argument
- changing stable JSON field names or meanings
- widening CLI dependencies into direct domain/stage/runner ownership
- changing public exports in [PUBLIC_API.md](PUBLIC_API.md)
- changing exit-code categories in `src/process_exit.rs`

## Non-Breaking Changes
- adding an optional flag with a default preserving current behavior
- adding a new command with clear ownership and tests
- improving human-readable wording while preserving command semantics
- adding fields to JSON when consumers can safely ignore them
- moving implementation behind `bijux-dna-api` while keeping CLI behavior stable

## Required Updates
- Command parser change: update [COMMANDS.md](COMMANDS.md).
- Public Rust surface change: update [PUBLIC_API.md](PUBLIC_API.md) and the public-surface snapshot.
- Layout change: update [ARCHITECTURE.md](ARCHITECTURE.md) and `tests/boundaries/architecture_tree.rs`.
- Output/help change: update [OUTPUT_FORMATS.md](OUTPUT_FORMATS.md) and snapshots.
- Dependency change: update [BOUNDARY.md](BOUNDARY.md), dependency policies, and architecture docs.
- Docs layout change: keep one root `README.md` and exactly ten docs under `docs/`.

## Verification
Run the narrowest relevant command first, then the full crate test when the change crosses command,
layout, or dependency boundaries:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --no-default-features
```
