# bijux-dna-dev Change Rules

Use these rules when changing this crate.

## Command changes
- Add or remove command ids in `src/catalog` first.
- Update `docs/COMMANDS.md` in the same change set.
- Add or update tests when command ids, routing, or command side effects change.

## Documentation changes
- Keep only `README.md` at the crate root.
- Keep crate docs under `docs/`, with no more than ten Markdown files.
- Update `docs/INDEX.md` whenever adding or removing a document.

## Source layout changes
- Keep `cli/` limited to parsing, routing, and user-facing reporting.
- Keep command side effects under `commands/`.
- Keep workspace discovery and process adapters under `runtime/`.
- Update `docs/ARCHITECTURE.md` and `tests/boundaries/architecture.rs` with any tree change.

## Dependency changes
- Prefer workspace-managed dependency versions.
- Re-check `docs/DEPENDENCIES.md` before adding a dependency.
- Do not introduce dependency edges from production runtime crates into this crate.
