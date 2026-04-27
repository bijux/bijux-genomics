# Science

`science/` is the authored and compiled science control surface for `bijux-genomics`.

## Purpose

- keep evidence, claims, reasoning, decisions, and bindings reviewable
- compile those authored records into deterministic traceability outputs
- freeze release bundles without hand-editing generated science state

## Authority Split

- [science/specs/data/README.md](specs/data/README.md) is the authored data-plane surface
- [science/specs/evidence/README.md](specs/evidence/README.md) is the authored evidence-plane surface
- [science/specs/releases/README.md](specs/releases/README.md) is the authored release-manifest surface
- [science/specs/reports/README.md](specs/reports/README.md) is the authored report-intent surface
- [science/specs/results/README.md](specs/results/README.md) is the authored result-plane surface
- `science/generated/**` is compiler output
- `artifacts/science-releases/**` is release output
- [science/docs/README.md](docs/README.md) is the local manual archive for non-shareable evidence payloads

The first implemented slice is the FASTQ environment and container support surface:

- which repo files are authoritative for admitted FASTQ stage tools
- which tool is the governed default for each FASTQ stage
- which planned tools remain outside the closed runtime surface
- which container and runtime references back each admitted tool
- which upstream source packets and paper roots back each reviewed FASTQ tool

This control surface does not replace `domain/**`, `configs/**`, `containers/**`, or
`crates/bijux-dna-environment/**`. It traces and compiles the claims that explain how those
surfaces are used.

## Generated Index

[science/generated/indexes/science_index.json](generated/indexes/science_index.json)
is the top-level operator entrypoint for the generated FASTQ science slice.

- Row counts show the size of each governed evidence surface.
- `source_archive_summary` shows which kinds of sources are present, which access modes they use,
  whether any governed archive payloads are missing, and which tool families would be blocked.
- `fastq_closure_summary` shows how many FASTQ bindings are world-class closed, declared closed
  with gaps, or still not closed, plus the rolled-up blocker and warning reasons.
- `fastq_evidence_summary` shows the distribution of backlog, paper archive, prerequisite, risk,
  and truth-delta categories without reopening every TSV.

Use the index to decide which evidence table to inspect next, then use the TSV
files under [science/generated/current/evidence/](generated/current/evidence/)
for the stage- and tool-level detail.

## Local Evidence Archive

`science/docs/` is intentionally separate from `science/`.

- `science/specs/**` records authored identifiers, claims, and reviewable metadata
- `science/docs/**` stores the local downloaded or cloned payloads that back those
  identifiers when redistribution is not acceptable
- the science compiler may report expected archive paths, but the archive contents
  themselves stay outside Git
