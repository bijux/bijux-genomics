# POLICY_STABILITY

## What
Defines stability expectations for policy IDs and behavior.

## Why
Stable policy identifiers make enforcement and reviews reproducible.

## Non-goals
- Describing every policy in detail.

## Contracts
Policy IDs are frozen. Changing semantics requires a changelog entry.

## Examples
- Renaming a policy requires a new ID and deprecation notice.

## Failure modes
- Silent changes break expectations and historical audits.
