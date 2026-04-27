# NO_ORPHANS

## What
Rule: every reference page must be linked from [REFERENCE_INDEX.md](REFERENCE_INDEX.md).

## Why
Prevents silent drift and orphaned documentation.

## Non-goals
- Replacing link checker output.

## Contracts
Every reference page must be linked from [REFERENCE_INDEX.md](REFERENCE_INDEX.md).
Broken links fail CI via docs link checks.

## Examples
- Add new reference pages to [REFERENCE_INDEX.md](REFERENCE_INDEX.md) in the same PR.

## Failure modes
- Orphaned references are skipped by review and tooling.
