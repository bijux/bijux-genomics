# Stage Taxonomy (BAM)

## What
Defines the stage grouping used for BAM pipelines: phase, class, and support status.

## Why
Planning, reporting, and coverage rules depend on consistent stage classification.

## Non-goals
- Listing every possible tool implementation.
- Defining execution details for each stage.

## Contracts
- Every stage must have a phase and class.
- Support status must be explicit and auditable.

## Examples
| Stage | Phase | Class | Support |
| --- | --- | --- | --- |
| align | pre | Essential | supported |
| sort | core | Essential | supported |
| index | core | Essential | supported |
| markdup | core | Recommended | supported |
| damage | downstream | Optional | supported |
| contamination | downstream | Optional | supported |
| authenticity | downstream | Optional | supported |
| kinship | downstream | Optional | not supported yet |

## Failure modes
- Missing stages or ambiguous classes break planner and report assumptions.
