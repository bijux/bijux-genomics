# ADD_OBSERVER

## Requirements
- Add fixtures under tests/fixtures/observer.
- Add determinism snapshot test.
- Update STAGE_CONTRACTS with canonical examples.
- Ensure parser emits canonical JSON.

## Checklist: add a new tool output parser
- Add fixtures under `tests/fixtures/*` for the tool output.
- Implement parser in `src/observer/parse.rs` (or a new module).
- Add snapshot coverage under `tests/observer/observer_parsers.rs`.
- Ensure canonical JSON via core canonicalizer.
