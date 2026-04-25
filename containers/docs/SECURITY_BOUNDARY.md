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
- `cargo run -p bijux-dna-dev -- containers run check-runtime-downloads`
- `cargo run -p bijux-dna-dev -- containers run check-network-disclosure`
- `cargo run -p bijux-dna-dev -- containers run check-no-secrets`
