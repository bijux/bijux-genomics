# Defaults Ledger

The defaults ledger records the effective defaults for each canonical pipeline profile. It is contract data, not a generated convenience note.

## Contents
- Pipeline ID.
- Stage order.
- Tool selections.
- Parameter defaults.
- Default provenance.
- Profile-specific overrides.

## Override Semantics
Precedence is:

1. Profile override.
2. Pipeline default.
3. Global default.

Overrides must be explicit. The crate must not invent implicit fallback defaults during merge.

Example:

```text
global trim.min_len = 20
pipeline trim.min_len = 25
profile trim.min_len = 30
effective trim.min_len = 30
```

## Change Rules
- Update the owning profile/default module in `src/`.
- Re-run the defaults and profile snapshot tests.
- Review snapshots for scientific and contract intent, not just byte changes.
- Version the pipeline ID when changed defaults alter expected outputs or scientific interpretation.

## Test Coverage
- `tests/contracts/defaults.rs` protects merge behavior, defaults ledger projection, and override precedence.
- `tests/contracts/profiles.rs` protects profile contracts and snapshots.
- `tests/contracts/registry.rs` protects registry inclusion and deterministic ordering.
