# POLICY_STABILITY

## What
Defines stability expectations for policy IDs and behavior.

## Why
Stable policy identifiers make enforcement and reviews reproducible.

## Non-goals
- Describing every policy in detail.

## Contracts
Policy IDs listed in [POLICY_INDEX.md](POLICY_INDEX.md) are frozen.
Changing semantics requires the same deliberate versioning posture documented in
[CONTRACT_VERSIONING.md](../50-reference/CONTRACT_VERSIONING.md).

## Examples
- Renaming a policy requires a new ID and deprecation notice.

## Failure modes
- Silent changes break expectations and historical audits.
