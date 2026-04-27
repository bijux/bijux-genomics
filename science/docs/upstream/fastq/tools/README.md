# FASTQ Tool Evidence Packets

This directory is the tracked instruction surface for manual FASTQ tool evidence
packets.

## Purpose

- reserve durable archive paths under `science/docs/upstream/fastq/tools/<tool-id>/`
- track the primary upstream and supplemental evidence locators for review
- complement [../README.md](../README.md),
  [../../README.md](../../README.md), and the operator backlog in
  [../../../TODO_DOWNLOAD.md](../../../TODO_DOWNLOAD.md) with tool-specific
  acquisition notes

## Canonical Files

- [EVIDENCE_MAP.tsv](EVIDENCE_MAP.tsv)
  tracked map of primary upstreams, supporting locators, and expected archive
  packet shape for focused FASTQ tools plus explicit contextual packets such as
  FastQ Screen when they still anchor governed FASTQ QC evidence
- [../../papers/TOOL_PAPER_MAP.tsv](../../papers/TOOL_PAPER_MAP.tsv)
  tracked map from FASTQ tools to durable paper archive roots

## Local Payloads

Keep payloads untracked and place them under the archive path declared in the
science backlog, for example:

- [<tool-id>/repo/](<tool-id>/repo/)
- [<tool-id>/download/](<tool-id>/download/)

When a tool needs both a source repository and a paper or release page, keep
the tool packet and the linked paper root aligned through
[EVIDENCE_MAP.tsv](EVIDENCE_MAP.tsv) and
[../../papers/TOOL_PAPER_MAP.tsv](../../papers/TOOL_PAPER_MAP.tsv) rather than
inventing a new root-level location.
