# POLICY_OWNERSHIP

## What
Defines where policies live and how other docs should reference them.

## Why
Single ownership avoids drift and conflicting guidance.

## Non-goals
- Describing policy implementation details.

## Contracts
Policies live only in `bijux-policies`.
Other docs must link to policy IDs, not redefine them.

## Examples
- Use `policy__surface__docs_spine__...` in docs instead of rewriting rules.

## Failure modes
- Duplicate policy descriptions diverge and cause inconsistent enforcement.
