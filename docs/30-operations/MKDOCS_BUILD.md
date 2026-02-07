# MkDocs Build

## What
MkDocs is the single source build for root docs.

## Why
Broken links must fail CI.

## Non-goals
- Building crate docs (they are linked only).

## Contracts
CI runs `mkdocs build --strict` and fails on broken links.

## Examples
```bash
make docs
make docs-lint
```

## Failure modes
Broken links or missing files fail the build.
