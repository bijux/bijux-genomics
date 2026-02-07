# Security

## What
Security practices for pipeline execution and data handling.

## Why
Protects user data and infrastructure.

## Non-goals
- Compliance certification.

## Contracts
- No untrusted code execution in non‑runner crates.

## Examples
- Runner is the only crate allowed to spawn processes.

## Failure modes
- Policy tests fail if forbidden APIs are imported.
