# Stage Taxonomy (FASTQ)

## What
Defines the FASTQ stage classes and support status used by planners and reports.

## Why
Stage classification controls completeness checks and pipeline guarantees.

## Non-goals
- Tool execution details.
- Exhaustive tool catalogs.

## Contracts
- Stage inventory and canonical FASTQ stage IDs live in
  [domain/fastq/index.yaml](../../../domain/fastq/index.yaml).
- Execution status and admitted support live in
  [domain/fastq/execution_support.yaml](../../../domain/fastq/execution_support.yaml).
- Readable stage descriptions live in [STAGE_CATALOG.md](STAGE_CATALOG.md).

## Examples
| Stage | Class | Support |
| --- | --- | --- |
| fastq.validate_reads | Essential | supported |
| fastq.trim_reads | Essential | supported |
| fastq.merge_pairs | Recommended | supported |
| fastq.filter_reads | Recommended | supported |
| fastq.screen_taxonomy | Optional | supported |
| fastq.report_qc | Optional | supported |
| fastq.profile_reads | Optional | supported |
| fastq.correct_errors | Optional | supported |
| fastq.extract_umis | Optional | supported |
| fastq.trim_terminal_damage | Optional | supported |

## Failure modes
- Missing classification breaks policy and planner checks.
