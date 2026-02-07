# bijux-pipelines architecture

## Role
`bijux-pipelines` is the source of truth for pipeline profiles and defaults ledgers. It bridges
domain contracts (FASTQ/BAM) to planner inputs without embedding tool execution logic.

## Dependencies
- Allowed: `bijux-core`, `bijux-domain-fastq`, `bijux-domain-bam`
- Forbidden: execution crates (`bijux-runtime`, `bijux-engine`, `bijux-runner`), stages, or
  environment backends.

## Stability
Profiles and defaults ledgers are snapshot-tested. Any changes must be intentional and reviewed.
