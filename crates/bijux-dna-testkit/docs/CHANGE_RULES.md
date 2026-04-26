# Change Rules

Use these rules when changing `bijux-dna-testkit`.

## Public API

- Removing, renaming, or changing the behavior of a root re-export is breaking.
- Adding a public helper requires docs, tests, and public API snapshot updates.
- Keep `src/public_api/surface.rs` aligned with `src/lib.rs` root re-exports.

## Test Helpers

- Helpers must stay domain-neutral and reusable across crates.
- Path helpers must reject absolute paths and parent traversal when deriving
  contained test paths.
- Snapshot helpers must normalize unstable host, temp, user, timestamp, and
  duration values deterministically.
- Fixture helpers must report failing paths in panic messages.

## Documentation

- Keep one root `README.md`.
- Keep all other crate docs in `docs/`.
- Keep `docs/COMMANDS.md` as the SSOT for callable operations.
- Keep `docs/` at 10 Markdown files or fewer.

## Dependencies

- Do not add production workspace crate dependencies.
- Keep `bijux-dna-policies` as a dev dependency only.
- Add dependency-boundary tests with any dependency graph change.
