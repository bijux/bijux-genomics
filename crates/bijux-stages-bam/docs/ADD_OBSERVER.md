# ADD_OBSERVER

## Requirements
- Add fixtures under tests/fixtures/observer.
- Add determinism snapshot test.
- Update STAGE_CONTRACTS with canonical examples.
- Ensure parser emits canonical JSON.

## Checklist: add a new observer
- Add fixtures under `tests/fixtures/*`.
- Implement parser in `src/observer.rs`.
- Add snapshot coverage under `tests/observer/observer_snapshots.rs`.
- Update stage contract snapshots and registry completeness tests.
