# Exceptions

## What
Documents allowlists and why they exist.

## Why
Explicit exceptions prevent policy drift and hidden coupling.

## Non-goals
- Permanent blanket exemptions.

## Contracts
Each exception must include:
- **Rule**: policy name
- **Reason**: why the exception is required
- **Owner**: responsible maintainer
- **Expiry**: date or “never”

## Examples
- `serde_yaml` allowlist in a single infra crate with documented rationale.

## Failure modes
- Exceptions without owner or expiry are rejected.

## Current exceptions
- None.
