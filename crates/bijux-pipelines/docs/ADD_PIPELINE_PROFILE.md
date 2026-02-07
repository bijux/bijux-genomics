# ADD_PIPELINE_PROFILE

Checklist for adding a new pipeline profile.

## Update registry
- Add the profile builder in the appropriate module (`fastq/`, `bam/`, or `cross/`).
- Register it in `src/registry/mod.rs` and any `*_profiles_by_id` helper.

## Defaults ledger
- Add defaults to `defaults_ledger.json` with stable ordering.
- Ensure `defaults/ledger.rs` remains canonical for ordering and JSON rules.

## Invariants
- Update any required stage/metrics/artifacts lists.
- Ensure profile capabilities are explicit (inputs, outputs, report sections).

## Snapshots + tests
- Add/update snapshots in `tests/snapshots/`.
- Extend `tests/registry/pipeline_registry_snapshot.rs` and `tests/profiles/pipeline_completeness.rs` if needed.
- Confirm override precedence in `tests/defaults/override_precedence.rs` if defaults changed.
