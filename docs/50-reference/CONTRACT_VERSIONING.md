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
- Contract snapshots must be updated alongside version bumps.

## Examples
- Adding a required field to RunManifest requires major bump and snapshot update.

## Failure modes
- Unversioned breaking changes fail policy.
