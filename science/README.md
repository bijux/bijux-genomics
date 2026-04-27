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

- [domain/fastq/execution_support.yaml](../domain/fastq/execution_support.yaml)
  records the admitted FASTQ stage-tool surface.
- [FASTQ_CONTAINER_DEFAULT_MATRIX.tsv](docs/upstream/fastq/container/FASTQ_CONTAINER_DEFAULT_MATRIX.tsv)
  materializes the governed default tool for each admitted FASTQ stage.
- [PLANNED_RUNTIME_BLOCKERS.tsv](docs/upstream/fastq/PLANNED_RUNTIME_BLOCKERS.tsv)
  tracks planned tools that remain outside the closed runtime surface.
- [FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv](docs/upstream/fastq/container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv)
  rolls up the container and runtime references that back each admitted default.
- [EVIDENCE_MAP.tsv](docs/upstream/fastq/tools/EVIDENCE_MAP.tsv) and
  [TOOL_PAPER_MAP.tsv](docs/upstream/papers/TOOL_PAPER_MAP.tsv) track the
  upstream source packets and paper roots behind reviewed FASTQ tools.

This control surface does not replace FASTQ domain manifests such as
[domain/fastq/execution_support.yaml](../domain/fastq/execution_support.yaml),
configuration indexes such as [configs/index.md](../configs/index.md),
operational container contracts in [containers/README.md](../containers/README.md),
or the runtime resolution surface in
[crates/bijux-dna-environment/README.md](../crates/bijux-dna-environment/README.md).
It traces and compiles the claims that explain how those surfaces are used.

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

[science/docs/README.md](docs/README.md) is intentionally separate from
`science/`.

- [science/specs/evidence/README.md](specs/evidence/README.md) records the
  authored identifiers, claims, and reviewable metadata for the current slice.
- [science/docs/TODO_DOWNLOAD.md](docs/TODO_DOWNLOAD.md) and
  [science/docs/upstream/README.md](docs/upstream/README.md) describe the local
  downloaded or cloned payloads that back those identifiers when redistribution
  is not acceptable.
- [science/generated/indexes/science_index.json](generated/indexes/science_index.json)
  may report expected archive paths, but the archive contents themselves stay
  outside Git.
