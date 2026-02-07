# CONTRACT

## Compatibility matrix
| Contract | Planner | Runtime | Analyze |
| --- | --- | --- | --- |
| v1 | supported | supported | supported |

## Breaking change definition
- removing fields
- renaming fields
- changing semantics
- Any breaking change requires a major version bump to the contract.

## No execution detail
This crate defines planning contracts only; execution belongs in core/runtime.
For execution manifests and run contracts, `bijux-core` is the authority.

## Tiny-crate promise
Non-goals: execution, IO, tool selection.
Checklist: no new modules without policy update.
