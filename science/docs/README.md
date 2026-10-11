# Science Docs Archive

`science/docs/` contains tracked archive contracts, locator tables, and acquisition
instructions. Downloaded papers, upstream repositories, compressed exports, and
tool-source packets live outside the product checkout, in the sibling directory
`../bijux-genomics-science/` by default.

Existing `archive_path`, `paper_root`, and mirror locator fields retain the
logical `science/docs/` namespace. Remove that prefix and resolve the remaining
path below the external archive. For example,
`science/docs/upstream/papers/<paper-id>/original/` resolves to
`../bijux-genomics-science/upstream/papers/<paper-id>/original/`.
The archive's `archive-inventory.json` preserves original paths and payload hashes.
Do not recreate product-local payload directories or compatibility symlinks.

## Purpose

- keep non-shareable science evidence near the repository without publishing it
- give science records stable locators independent of external payload storage
- preserve a durable folder contract for later reviews, audits, and release work

## What Belongs In The External Archive

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

- keep actual evidence payloads outside the product checkout
- keep durable identifiers and download instructions in [TODO_DOWNLOAD.md](TODO_DOWNLOAD.md)
- treat [fastq_download_backlog.tsv](../generated/current/evidence/fastq_download_backlog.tsv) as the
  machine-readable source for FASTQ archive planning and keep
  [TODO_DOWNLOAD.md](TODO_DOWNLOAD.md) aligned with it
- use [science/generated/current/evidence/README.md](../generated/current/evidence/README.md)
  when a review needs the wider generated evidence inventory behind that backlog
- resolve each planned archive locator against the external root before acquisition
- use `--evidence-root` or `BIJUX_SCIENCE_ARCHIVE_ROOT` for an alternate external root
  with the GitHub sync command; mirror and export overrides must stay beneath it
- normal science validation and generation consume tracked declarations and do not
  require the local payload archive or infer fresh acquisition from its presence

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

GitHub payloads use `upstream/github-repos/` below the external archive root.
Their tracked instructions and manifest remain under `science/docs/upstream/github-repos/`.
