# ENV_REFERENCE

## Resolution Precedence
1. Explicit image digest
2. Versioned image tag
3. Default tool image

## Digest Rules
If a digest is provided, it is authoritative and must not be overridden.

## Caching
Resolved images are cached by digest + platform.

## Determinism
Given identical specs, resolution must return the same digest and version.

## Fixtures
See `tests/reference_matrix.rs` for the curated fixture matrix and expectations.
