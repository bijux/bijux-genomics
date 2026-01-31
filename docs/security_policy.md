# Security Policy

## RustSec advisory allowlist

We use `cargo-deny` to enforce advisories. Any allowlisted advisory must be documented in `deny.toml` with a rationale and reviewed during dependency upgrades. Currently, the allowlist is empty; if an advisory is temporarily allowed, add it to `deny.toml` with a clear reason and an exit criteria.

## Dependency upgrade tracking (arrow/parquet)

The `parquet` and `arrow` dependency family is tracked explicitly due to their size and security surface.

- Review cadence: every dependency upgrade sweep.
- Owners: platform/infra.
- Upgrade notes must include:
  - new version
  - RustSec status
  - any API changes affecting metrics loading

## Reporting

If a security advisory is discovered during CI, open an issue with:
- advisory ID
- impacted crates
- remediation plan and timeline
