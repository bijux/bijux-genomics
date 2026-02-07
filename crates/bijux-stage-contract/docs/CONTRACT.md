# CONTRACT

## Compatibility matrix
| Contract | Planner | Runtime | Analyze |
| --- | --- | --- | --- |
| v1 | supported | supported | supported |

## Breaking change definition
- removing fields
- renaming fields
- changing semantics

## No execution detail
This crate defines planning contracts only; execution belongs in core/runtime.

## Tiny-crate promise
Non-goals: execution, IO, tool selection.
Checklist: no new modules without policy update.
