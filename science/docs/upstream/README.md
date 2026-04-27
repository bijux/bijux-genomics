# Science Docs Upstream Archive

`science/docs/upstream/` is the canonical local archive root for upstream source
material used to support science, container, and provenance review in
`bijux-genomics`.

[../README.md](../README.md) defines the broader local archive boundary, and
[../TODO_DOWNLOAD.md](../TODO_DOWNLOAD.md) remains the operator-facing backlog
for manual acquisition outside the GitHub mirror workflow.

## Scope

- GitHub repository mirrors
- manual upstream release downloads
- local snapshots of upstream source material that should stay outside Git

## Layout

- [PRODUCTION_CLOSURE_INDEX.tsv](PRODUCTION_CLOSURE_INDEX.tsv)
  tracked rollup pointing at the FASTQ, BAM, VCF, reference-asset, and primer-evidence closure authorities
- [bam/README.md](bam/README.md)
  contract for BAM citation and promotion blockers
- [bam/BAM_PRODUCTION_CLOSURE_LEDGER.tsv](bam/BAM_PRODUCTION_CLOSURE_LEDGER.tsv)
  tracked BAM closure blocker ledger
- [fastq/README.md](fastq/README.md)
  contract for FASTQ upstream evidence packets
- [fastq/tools/README.md](fastq/tools/README.md)
  per-tool archive rules for FASTQ source material
- [fastq/tools/EVIDENCE_MAP.tsv](fastq/tools/EVIDENCE_MAP.tsv)
  tracked locator map for tool-specific manual archive packets
- [papers/README.md](papers/README.md)
  contract for local paper archive roots
- [papers/TODO_DOWNLOAD.md](papers/TODO_DOWNLOAD.md)
  operator-facing paper archive worklist
- [papers/TOOL_PAPER_MAP.tsv](papers/TOOL_PAPER_MAP.tsv)
  tracked tool-to-paper root map
- [vcf/README.md](vcf/README.md)
  contract for VCF calling and downstream promotion blockers
- [vcf/VCF_DOWNSTREAM_CLOSURE_LEDGER.tsv](vcf/VCF_DOWNSTREAM_CLOSURE_LEDGER.tsv)
  tracked VCF closure blocker ledger
- [github-repos/README.md](github-repos/README.md)
  contract for GitHub repository evidence mirrors
- [github-repos/MANIFEST.tsv](github-repos/MANIFEST.tsv)
  tracked repository target manifest
- `github-repos/mirrors/**`
  untracked local bare clones
- `github-repos/archives/**`
  optional untracked compressed exports

## Rules

- keep the archive payloads untracked
- keep the target list and archive contract tracked
- prefer one stable upstream location per evidence family instead of ad hoc
  directories at the root of `science/docs/`
- keep FASTQ, BAM, and VCF closure authorities separate so one domain does not
  inherit release status from another
