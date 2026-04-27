# DOCS_STYLE

## Index Rules
- Every major docs section must have an index file: `docs/<section>/index.md`.
- Root docs index is [docs/index.md](../index.md).
- Section graph is maintained in [docs/00-intro/DOCS_MAP.md](../00-intro/DOCS_MAP.md).
- Executable graph contract is maintained in [docs/DOCS_GRAPH.toml](../DOCS_GRAPH.toml) and
  enforced by docs checks.

## Naming Conventions
- Use UPPER_SNAKE_CASE for top-level policy/reference docs where already established.
- Use `index.md` for section indexes.
- Keep filenames stable and descriptive; avoid ad-hoc aliases.

## Generated Docs
- Generated docs must use `.generated.md` suffix when appropriate.
- Generated docs must include the standard header:
  - `<!-- GENERATED FILE - DO NOT EDIT -->`
  - `<!-- Regenerate with: <script> -->`
- Manual edits to generated docs are forbidden; update via generator scripts.

## Doc Depth Rules (Major Docs)
- Major docs (authority/operations/reference entry docs) must include:
  - `## Purpose`
  - `## Scope`
  - `## Contracts`
  - `## Examples`
  - `## Failure modes`
- Section-specific expectations:
  - `docs/10-architecture/*`: explicit authority and SSOT references.
  - `docs/20-science/*`: scientific assumptions and validity bounds.
  - `docs/30-operations/*`: run/CI/isolation/HPC operational behavior.
  - `docs/40-policies/*`: enforceable policy statements and gates.
  - `docs/50-reference/*`: canonical contracts and workflow checklists.

## Contract Authority Ladder
- See [docs/10-architecture/CONTRACT_AUTHORITY_LADDER.md](../10-architecture/CONTRACT_AUTHORITY_LADDER.md)
  for authoritative precedence.
- Docs are normative for process, but generated/config/domain/container sources remain authoritative for executable contracts.

## Purpose
This document defines the intended behavior and navigation contract for this topic.

## Scope
Applies only to the files and workflows referenced in this document.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.
