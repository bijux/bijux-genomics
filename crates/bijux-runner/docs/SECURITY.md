# SECURITY

## Boundaries
- Docker execution is constrained to explicit mounts and allowlisted env vars.
- Local execution runs with a restricted working directory.

## Secrets
Secrets must never be logged or written to manifests. Redaction is mandatory.
