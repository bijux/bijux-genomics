# Science Docs Upstream Archive

`science-docs/upstream/` is the canonical local archive root for upstream source
material used to support science, container, and provenance review in
`bijux-genomics`.

## Scope

- GitHub repository mirrors
- manual upstream release downloads
- local snapshots of upstream source material that should stay outside Git

## Layout

- `fastq/README.md`
  contract for FASTQ upstream evidence packets
- `fastq/tools/README.md`
  per-tool archive rules for FASTQ source material
- `fastq/tools/EVIDENCE_MAP.tsv`
  tracked locator map for tool-specific manual archive packets
- `papers/README.md`
  contract for local paper archive roots
- `papers/TODO_DOWNLOAD.md`
  operator-facing paper archive worklist
- `papers/TOOL_PAPER_MAP.tsv`
  tracked tool-to-paper root map
- `github-repos/README.md`
  contract for GitHub repository evidence mirrors
- `github-repos/MANIFEST.tsv`
  tracked repository target manifest
- `github-repos/mirrors/**`
  untracked local bare clones
- `github-repos/archives/**`
  optional untracked compressed exports

## Rules

- keep the archive payloads untracked
- keep the target list and archive contract tracked
- prefer one stable upstream location per evidence family instead of ad hoc
  directories at the root of `science-docs/`
