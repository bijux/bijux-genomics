# Contract Versioning

## What
Rules for contract version bumps.

## Why
Prevents silent breaking changes.

## Non-goals
- Automatic versioning.

## Contracts
- Breaking change ⇒ major bump.
- Additive change ⇒ minor bump.
- Contract snapshots must be updated alongside version bumps under
  [SCHEMAS_INDEX.md](SCHEMAS_INDEX.md).
- Compatibility disclosures must stay aligned with
  [COMPATIBILITY_MATRIX.md](COMPATIBILITY_MATRIX.md).

## Examples
- Adding a required field to RunManifest requires major bump and snapshot update.

## Failure modes
- Unversioned breaking changes fail policy.
