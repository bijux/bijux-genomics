# Bless Workflow

## What
Defines when and how snapshot updates are allowed ("bless" flow).

## Why
Snapshots are contracts; unrestricted updates hide regressions.

## Non-goals
- Auto-accepting snapshots in CI.

## Contracts
- Only update snapshots when the change is intentional and reviewed.
- Snapshot updates must accompany the policy/doc change that caused them.
- Never bless snapshots to silence failures unrelated to intentional changes.

## Examples
- Adding a new allowed doc file updates `crate_docs_tree_contract.snap`.
- Introducing a new policy test updates its snapshot alongside the test.

## Failure modes
- Unreviewed snapshot updates obscure contract regressions.
- Blessing without a policy change is treated as a violation.
