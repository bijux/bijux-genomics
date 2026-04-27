# Upstream Paper Archive

`science/docs/upstream/papers/` is the tracked contract surface for local paper
roots that anchor tool evidence packets to publications.

[../README.md](../README.md) defines the broader upstream archive boundary that
this paper packet surface specializes.

## Purpose

- give each cited tool paper a durable local root under `science/docs/`
- let tool-source packets point at paper roots without embedding PDFs in Git
- support incremental completion from tool claims to publication evidence

## Canonical Files

- [TODO_DOWNLOAD.md](TODO_DOWNLOAD.md)
  operator-facing paper archive worklist for local downloads and homepage snapshots
- [TOOL_PAPER_MAP.tsv](TOOL_PAPER_MAP.tsv)
  tracked map from tools to durable paper roots, access status, and paper locators

## Local Payloads

Keep paper payloads untracked. For each `paper_root`, place downloaded material
under the matching directory, for example:

- [<paper-id>/original/](<paper-id>/original/)
- [<paper-id>/notes/](<paper-id>/notes/)

Use the paper root even when the paper is not yet downloaded or requires
licensed access. The root itself is part of the contract.
