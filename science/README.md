# Science

`science/` is the authored and compiled science control surface for `bijux-genomics`.

## Purpose

- keep evidence, claims, reasoning, decisions, and bindings reviewable
- compile those authored records into deterministic traceability outputs
- freeze release bundles without hand-editing generated science state

## Authority Split

- `science/specs/**` is human-authored review input
- `science/generated/**` is compiler output
- `artifacts/science-releases/**` is release output
- `science/docs/**` is a local manual archive for non-shareable evidence payloads

The first implemented slice is the FASTQ environment and container support surface:

- which repo files are authoritative for admitted FASTQ stage tools
- which tool is the governed default for each FASTQ stage
- which planned tools remain outside the closed runtime surface
- which container and runtime references back each admitted tool
- which upstream source packets and paper roots back each reviewed FASTQ tool

This control surface does not replace `domain/**`, `configs/**`, `containers/**`, or
`crates/bijux-dna-environment/**`. It traces and compiles the claims that explain how those
surfaces are used.

## Local Evidence Archive

`science/docs/` is intentionally separate from `science/`.

- `science/specs/**` records authored identifiers, claims, and reviewable metadata
- `science/docs/**` stores the local downloaded or cloned payloads that back those
  identifiers when redistribution is not acceptable
- the science compiler may report expected archive paths, but the archive contents
  themselves stay outside Git
