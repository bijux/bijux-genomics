# DEFAULTS_LEDGER

## What the ledger contains
- Effective defaults for each pipeline profile.
- Tool selections and provenance for defaults.
- Thresholds and parameter provenance.

## How to change defaults
- Update the pipeline/profile definition in `src/*`.
- Re-run the defaults ledger snapshot tests.
- Review diff to confirm intentional changes.

## Snapshot protection
Snapshots in `tests/defaults/defaults_ledger.rs` protect the ledger contract.
Update only for intentional contract changes.

## Override semantics
Precedence: profile > pipeline > global.

Example:
- profile sets `trim_min_len` and overrides the pipeline default.

See `tests/defaults/override_precedence.rs`.
