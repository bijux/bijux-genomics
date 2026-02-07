# Threat Model

## What
Defines what is executed and what is never executed.

## Why
Prevent untrusted execution and ensure safe replay.

## Non-goals
- Comprehensive security audit.

## Contracts
- Runner executes tools.
- Replay never executes tools (validated by runner tests).

## Examples
- Execution: only via runner backends.
- Replay: validates artifacts only.

## Failure modes
Any replay execution attempt is a policy violation.
