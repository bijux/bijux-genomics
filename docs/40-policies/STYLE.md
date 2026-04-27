# Style

## What
Documentation and code style rules for Bijux Genomics.

## Why
Consistency reduces review time and errors.

## Non-goals
- Enforcing personal formatting preferences.

## Contracts
- Docs placement contract is defined in [DOCS_STYLE.md](DOCS_STYLE.md) and enforced by
  [boundary_docs_policy.rs](../../crates/bijux-dna-policies/tests/contracts/tooling/docs/boundary_docs_policy.rs).

## Examples
- Crate docs live under the boundary surfaces enforced by
  [boundary_docs_policy.rs](../../crates/bijux-dna-policies/tests/contracts/tooling/docs/boundary_docs_policy.rs).

## Failure modes
- Misplaced docs fail policies.

## Docs Depth Policy
- Every authored Markdown doc must include: `## Purpose`, `## Scope`, `## Non-goals`, and `## Contracts`.
- Transitional aliases are accepted temporarily by checks:
  - `## What` can satisfy `Purpose`
  - `## Why` can satisfy `Scope`
- Generated docs and `index.md` files are exempt.
