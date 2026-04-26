# Security

`bijux-dna-api` is a local Rust API crate. It does not implement authentication,
authorization, HTTP routing, or network transport. Callers are trusted in-process
code inside the `bijux-genomics` workspace or consumers that explicitly link the
crate.

## Trust Model

- Authentication and authorization must be handled by the application layer that
  calls this crate.
- This crate must not open network listeners or make network calls during API
  request handling.
- Public v1 responses must not expose secrets, tokens, credentials, or host-local
  environment details that are not already declared as run artifacts.

## Filesystem Rules

- Write only under caller-declared run, report, audit, or dry-run output roots.
- Read only workspace configuration, reference locks, registries, run artifacts,
  and declared input artifacts needed for the request.
- Include provenance hashes for declared artifacts when the workflow requires
  evidence.
- Do not create hidden global cache/state that changes output shape.

## Process And Container Rules

- Do not perform ad hoc process or container spawning from API modules.
- Execution must route through typed runner/runtime APIs.
- Dry-run, plan, status, explain, and policy-audit flows must not execute stage
  tools.

## Redaction

- Strip secrets and tokens from logs, reports, manifests, audit output, and
  operator-facing errors.
- Avoid serializing full environment snapshots unless a contract explicitly
  declares each field.
- Prefer stable artifact paths and hashes over verbose local machine details.

## Review Triggers

Request a security review when a change adds:

- A new filesystem write location.
- A new source of environment data in public responses.
- A new process/container execution path.
- A new public schema field that can contain operator, host, credential, or
  sample-identifying data.
