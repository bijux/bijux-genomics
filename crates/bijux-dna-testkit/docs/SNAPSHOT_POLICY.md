# Snapshot Policy

Snapshots and fixtures are contract inputs for the workspace test suites.

## Snapshot Rules

- Normalize host, temp, user, timestamp, duration, and artifact path values
  before asserting snapshots.
- Keep snapshot names stable through `snapshot_name(bucket, test_name)`.
- Install deterministic locale and timezone with `install_snapshot_env` when a
  test is sensitive to environment formatting.
- Bless snapshot changes only after reviewing the rendered diff.

## Fixture Rules

- Use the smallest artifact that still exercises the contract.
- Keep fixture files deterministic; avoid timestamps, random fields, secrets,
  and host-specific paths.
- If a timestamp or host path is required to exercise normalization, assert the
  normalized output explicitly.
- Fixture readers must include the failing path in panic messages.

## Naming Rules

- Prefer names that describe the contract and artifact type.
- Avoid disposable labels such as `tmp`, `misc`, or `wip`.
- Keep fixture and snapshot names durable enough to remain understandable in
  future reviews.
