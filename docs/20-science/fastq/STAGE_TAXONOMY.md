# Stage Taxonomy (FASTQ)

## What
Defines the FASTQ stage classes and support status used by planners and reports.

## Why
Stage classification controls completeness checks and pipeline guarantees.

## Non-goals
- Tool execution details.
- Exhaustive tool catalogs.

## Contracts
- Every stage must have a class and support status.
- Optional stages must be explicitly marked.

## Examples
| Stage | Class | Support |
| --- | --- | --- |
| validate_reads | Essential | supported |
| trim_reads | Essential | supported |
| merge | Recommended | supported |
| filter_reads | Recommended | supported |
| screen_taxonomy | Optional | supported |
| report_qc | Optional | supported |
| profile_reads | Optional | supported |
| correct | Optional | supported |
| umi | Optional | supported |
| damage_profile | Optional | not supported yet |

## Failure modes
- Missing classification breaks policy and planner checks.
