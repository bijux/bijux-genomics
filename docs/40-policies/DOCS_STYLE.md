# DOCS_STYLE

## Index Rules
- Every major docs section must have an index file: `docs/<section>/index.md`.
- Root docs index is `docs/index.md`.
- Section graph is maintained in `docs/00-intro/DOCS_MAP.md`.

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

## Purpose
This document defines the intended behavior and navigation contract for this topic.

## Scope
Applies only to the files and workflows referenced in this document.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.

