# Policy Diagnostics

## What
Standardizes policy failure messages so they are actionable and consistent.

## Why
Consistent diagnostics reduce triage time and prevent confusion.

## Non-goals
- Exhaustive troubleshooting guides.

## Contracts
Each policy failure must include:
1. **Symptom** — what failed.
2. **Rule** — the policy being enforced.
3. **Fix** — the specific change required.
4. **Example** — a short example of a compliant change.

## Examples
Symptom: crate docs missing `SCOPE.md`  
Rule: docs placement contract  
Fix: add `crates/<crate>/docs/SCOPE.md`  
Example: `crates/bijux-core/docs/SCOPE.md`

## Failure modes
- Policy output omits a required section.
