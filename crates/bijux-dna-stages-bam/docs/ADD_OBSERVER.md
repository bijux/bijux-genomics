# ADD_OBSERVER

## Requirements
- Add fixtures under `tests/fixtures/observer/default/`.
- Add determinism or snapshot coverage under `tests/contracts/observer/`.
- Update STAGE_CONTRACTS with canonical examples.
- Ensure parser emits canonical JSON.

## Checklist: add a new observer
- Add fixtures under `tests/fixtures/observer/default/`.
- Implement parser support in `bijux-dna-domain-bam` metrics and re-export it from `src/observer.rs`.
- Add snapshot coverage under `tests/contracts/observer/observer_snapshots.rs`.
- Update stage contract snapshots and registry completeness tests.
