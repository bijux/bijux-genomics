# Enforcement Workflow

## Policy Gates
- `make guardrails` checks per-crate guardrail configs and source-surface constraints.
- `make policies` runs workspace boundary, ownership, docs, dependency, and tooling policies.
- `make structure-check` verifies tree shape, docs spine, and layout contracts.

## Package Commands
Use `docs/COMMANDS.md` as the SSOT for commands this crate manages.

## Diagnostics Format
Policy failures should explain:

- WHAT failed.
- WHY the rule exists.
- HOW to repair the violation.
- MORE context, usually a crate doc or policy test path.

## Snapshot Changes
Snapshots are contracts. Update them only when the policy, fixture, or documentation contract intentionally changes, and keep the snapshot update in the same reviewable change as the cause.

## Policy Changes
- Adding a stricter policy is allowed when it has clear ownership and documentation.
- Weakening or deleting a policy requires an explicit boundary reason in the change.
- New allowlists must include a narrow reason and owner in code or docs.
