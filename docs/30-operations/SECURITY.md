# Security

## What
Operational security posture for Bijux runs.

## Why
Limits untrusted execution and data exposure.

## Non-goals
- Full threat modeling for all dependencies.

## Contracts
- Execution only via runner backends.
- Replay is non-executing.

## Examples
See [THREAT_MODEL.md](THREAT_MODEL.md) for explicit guarantees.

## Failure modes
Security violations trigger policy failures.
