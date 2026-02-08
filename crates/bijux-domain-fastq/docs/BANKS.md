# BANKS

## Authority
Bank content lives in `src/banks/*`. Downstream crates may select banks but must not redefine
or mutate bank contents.

## Provenance
- Adapter banks: curated adapter sequences used for trimming/detection.
- Contaminant banks: known contaminant sequences for screen stages.
- PolyX banks: polyG/polyN reference sets for artifact detection.

## Versioning rules
- Bank changes are contract changes. Update `docs/CHANGE_RULES.md` and refresh any snapshots.
- Add/remove entries only with an explicit provenance note in the corresponding `src/banks/*.rs`.
- Keep ordering stable; selection relies on deterministic bank hashes.

## Selection
Selection logic is centralized in `src/banks/selection.rs`. No other crate may override it.
