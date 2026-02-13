# Style

## What
Documentation and code style rules for Bijux DNA.

## Why
Consistency reduces review time and errors.

## Non-goals
- Enforcing personal formatting preferences.

## Contracts
- Docs placement contract.

## Examples
- Crate docs live in crates/<crate>/docs/.

## Failure modes
- Misplaced docs fail policies.

## Docs Depth Policy
- Every authored Markdown doc must include: `## Purpose`, `## Scope`, `## Non-goals`, and `## Contracts`.
- Transitional aliases are accepted temporarily by checks:
  - `## What` can satisfy `Purpose`
  - `## Why` can satisfy `Scope`
- Generated docs and `index.md` files are exempt.
