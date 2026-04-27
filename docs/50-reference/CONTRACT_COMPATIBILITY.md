# Contract Compatibility

## What
Compatibility rules for contract evolution.

## Why
Supports forward/backward compatibility.

## Non-goals
- Full migration tooling.

## Contracts
- New fields must be additive under [CONTRACT_VERSIONING.md](CONTRACT_VERSIONING.md).
- Compatibility statements must match [COMPATIBILITY_MATRIX.md](COMPATIBILITY_MATRIX.md).

## Examples
- Optional fields are safe additions.

## Failure modes
- Breaking field removal requires major bump.
