# Security Boundary

Purpose: clarify build-time vs runtime trust boundaries.

## Build-Time Boundary
- Network access may be required for pinned source acquisition.
- Every fetched source must be pinned by checksum or commit.
- Build scripts must emit provenance and lock metadata.

## Runtime Boundary
- Runtime should not perform implicit downloads unless explicitly declared.
- Runtime writes are restricted to allowed output roots.
- Smoke tests verify deterministic help/version/minimal behavior.

## Enforcement
- `scripts/containers/check-runtime-downloads.sh`
- `scripts/containers/check-network-disclosure.sh`
- `scripts/containers/check-no-secrets.sh`
