# Security Policy

## Scope

`bijux-analyze` processes pipeline facts and reports. Optional parquet/arrow support is gated behind the `parquet` feature to limit the default dependency surface.

## Advisory handling

- Run `cargo audit` during CI.
- If a temporary allowlist is required, document it in `audit-allowlist.toml` with:
  - advisory ID
  - rationale
  - expiry date
  - owner
- Review cadence: every dependency upgrade sweep (minimum quarterly).

## Reporting

Report security issues via the standard engineering escalation path. Include:
- advisory ID (if available)
- impacted crates
- remediation plan and timeline
