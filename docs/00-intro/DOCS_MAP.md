# DOCS_MAP

## What
Guide to reading and extending the documentation.

## Why
Ensures changes land in the correct location and stay consistent.

## Non-goals
- Detailed architecture (see `../10-architecture/ARCHITECTURE_OVERVIEW.md`).

## Contracts
Documentation placement is enforced by policy tests.

## Examples
Numbering scheme:
- `00-intro`: starting points and orientation.
- `10-architecture`: contracts, boundaries, SSOT.
- `20-science`: FASTQ/BAM scientific specs.
- `30-operations`: run artifacts and observability.
- `40-policies`: enforcement rules and workflows.
- `50-reference`: versioning, glossary, crate map.

If you change:
- **contracts** → read `10-architecture/CONTRACT_SPINE.md` first.
- **pipelines** → read `50-reference/PIPELINES.md` and `20-science/*`.
- **execution** → read `30-operations/RUN_ARTIFACTS.md`.
- **policies** → read `40-policies/POLICIES_EXPLAINED.md`.

Crate-specific docs live under `crates/<crate>/docs/`.

## Failure modes
Docs placed in the wrong tree fail CI policy checks.
