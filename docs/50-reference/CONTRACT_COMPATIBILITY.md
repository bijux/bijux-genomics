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
- Schema-family compatibility classes and migration rules must match [SCHEMA_REGISTRY.md](SCHEMA_REGISTRY.md).
- Route-level API adapter discipline must match [API_VERSIONING.md](API_VERSIONING.md).
- Durable operator and release-review failures must use the generated error registry section in [SCHEMA_REGISTRY.md](SCHEMA_REGISTRY.md).

## Examples
- Optional fields are safe additions.

## Failure modes
- Breaking field removal requires major bump.
