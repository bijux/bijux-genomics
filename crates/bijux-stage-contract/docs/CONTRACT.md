# CONTRACT

## Compatibility matrix
| Contract | Planner | Runtime | Analyze |
| --- | --- | --- | --- |
| v1 | supported | supported | supported |

## Breaking change definition
Breaking change = major bump. Examples:
- Removing fields
- Renaming fields
- Changing semantics

Tests under `tests/versioning/*` enforce that breaking changes require a major bump.

## Terminology
- **Plan**: a planned set of steps (this crate).
- **Run**: an executed plan with runtime artifacts (runtime/runner crates).
- **Execution plan**: the serialized plan JSON defined by this crate.

## No execution detail
This crate defines planning contracts only; execution belongs in core/runtime.
For execution manifests and run contracts, `bijux-core` is the authority.

## Example
See `docs/EXAMPLE_PLAN.json` with annotations in `docs/EXAMPLE_PLAN.md`.

## Tiny-crate promise
Non-goals: execution, IO, tool selection.
Checklist: no new modules without policy update.
