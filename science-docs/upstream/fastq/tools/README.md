# FASTQ Tool Evidence Packets

This directory is the tracked instruction surface for manual FASTQ tool evidence
packets.

## Purpose

- reserve durable archive paths under `science-docs/upstream/fastq/tools/<tool-id>/`
- track the primary upstream and supplemental evidence locators for review
- complement `science-docs/TODO_DOWNLOAD.md` with tool-specific acquisition notes

## Canonical Files

- `EVIDENCE_MAP.tsv`
  tracked map of primary upstreams, supporting locators, and expected archive
  packet shape for focused FASTQ tools

## Local Payloads

Keep payloads untracked and place them under the archive path declared in the
science backlog, for example:

- `science-docs/upstream/fastq/tools/<tool-id>/repo/`
- `science-docs/upstream/fastq/tools/<tool-id>/download/`

When a tool needs both a source repository and a paper or release page, keep
them under the same tool directory in separate subpaths rather than inventing a
new root-level location.
