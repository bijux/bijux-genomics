# Science Docs Archive

`science/docs/` is the local evidence archive for `bijux-genomics`.

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
- generated compiler outputs under [science/generated/README.md](../generated/README.md)
- release bundles owned by [science/specs/releases/README.md](../specs/releases/README.md)
  and cut under `artifacts/science-releases/**`

## Handling Rules

- keep actual evidence payloads untracked
- keep durable identifiers and download instructions in [TODO_DOWNLOAD.md](TODO_DOWNLOAD.md)
- treat [fastq_download_backlog.tsv](../generated/current/evidence/fastq_download_backlog.tsv) as the
  machine-readable source for FASTQ archive planning and keep
  [TODO_DOWNLOAD.md](TODO_DOWNLOAD.md) aligned with it
- use [science/generated/current/evidence/README.md](../generated/current/evidence/README.md)
  when a review needs the wider generated evidence inventory behind that backlog
- place each acquired item under a stable path that matches the planned archive
  path recorded in the TODO list or a future science source record

## Canonical Layout

- [science/docs/TODO_DOWNLOAD.md](TODO_DOWNLOAD.md)
  operator-facing backlog for manual papers, release bundles, and non-GitHub downloads
- [science/docs/upstream/README.md](upstream/README.md)
  upstream archive contract for local mirrors and source payloads
- [science/docs/upstream/fastq/README.md](upstream/fastq/README.md)
  FASTQ-specific archive contract for tool-source evidence packets
- [science/docs/upstream/fastq/tools/EVIDENCE_MAP.tsv](upstream/fastq/tools/EVIDENCE_MAP.tsv)
  tracked locator map for focused FASTQ tool evidence packets
- [science/docs/upstream/papers/README.md](upstream/papers/README.md)
  paper archive contract for tool-linked publications
- [science/docs/upstream/papers/TODO_DOWNLOAD.md](upstream/papers/TODO_DOWNLOAD.md)
  operator-facing paper archive backlog for local publication payloads
- [science/docs/upstream/papers/TOOL_PAPER_MAP.tsv](upstream/papers/TOOL_PAPER_MAP.tsv)
  tracked map from FASTQ tools to durable paper archive roots
- [science/docs/upstream/github-repos/README.md](upstream/github-repos/README.md)
  contract for GitHub repository evidence mirrors
- [science/docs/upstream/github-repos/MANIFEST.tsv](upstream/github-repos/MANIFEST.tsv)
  tracked manifest of GitHub repository evidence targets
- [science/docs/upstream/github-repos/README.md](upstream/github-repos/README.md)
  governs the untracked local bare clones used as the canonical GitHub repo archive
  plus optional compressed exports when a smaller portable snapshot is needed

Do not treat `science/docs/github-repos/` at the archive root as the canonical
shape going forward. The governed location for GitHub repository evidence is
`science/docs/upstream/github-repos/`.
