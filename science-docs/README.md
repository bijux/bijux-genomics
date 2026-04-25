# Science Docs Archive

`science-docs/` is the local evidence archive for `bijux-genomics`.

It exists for source material that is useful for review and validation but should
not be committed into Git, such as downloaded papers, cloned upstream
repositories, supplementary documents, and local notes tied to licensed access.

## Purpose

- keep non-shareable science evidence near the repository without publishing it
- give `science/` a stable local landing area for manually acquired evidence
- preserve a durable folder contract for later reviews, audits, and release work

## What Belongs Here

- manually downloaded papers
- manually cloned upstream repositories
- local evidence packets used to confirm science claims
- private or licensed material that may be accessed locally but not redistributed

## What Does Not Belong Here

- review-authored science SSOT under `science/specs/**`
- generated compiler outputs under `science/generated/**`
- release bundles under `artifacts/science-releases/**`

## Handling Rules

- keep actual evidence payloads untracked
- keep durable identifiers and download instructions in `TODO_DOWNLOAD.md`
- place each acquired item under a stable path that matches the planned archive
  path recorded in the TODO list or a future science source record
