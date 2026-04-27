# Policies Explained

## What
Explains why each policy exists.

## Why
Makes architectural rules understandable and enforceable.

## Non-goals
- Replacing policy tests.

## Contracts
- The governed catalog lives in [POLICY_INDEX.md](POLICY_INDEX.md).
- Coverage relationships live in [POLICY_MATRIX.md](POLICY_MATRIX.md).
- Fast remediation summaries live in [FAILURE_PLAYBOOKS.md](FAILURE_PLAYBOOKS.md).

## Examples
- No thin modules prevents single‑file directories.

## Failure modes
- CI fails when policies are violated.
