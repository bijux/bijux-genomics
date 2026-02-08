# Enforcement Workflow

## Golden workflow
Run all policy gates together:

`make guardrails && make policies && make structure-check`

## What each gate covers
- `make guardrails`: per-crate guardrail configs (public surface, module counts, etc.).
- `make policies`: workspace boundary, ownership, and doc policies.
- `make structure-check`: tree shape, docs spine, and layout contracts.

## How to debug failures
- Use the WHAT/WHY/HOW/MORE message to identify the rule and fix.
- Open the test file referenced in the failure message.
- Check `docs/TESTS.md` for the intent of the rule.

## Snapshot updates
Only bless snapshots when the policy change is intentional. See `docs/BLESS_WORKFLOW.md`.
