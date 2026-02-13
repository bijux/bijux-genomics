# Reference Source Lock

## Purpose
Define pinned upstream sources for reference assets and a safe update workflow.

## Pinned Sources
- `assets/reference/contaminants/references/univec.fasta`
  - upstream: NCBI UniVec snapshot
  - update method: explicit download to staging + checksum review
- `assets/reference/contaminants/references/phix174.fasta`
  - upstream: PhiX174 reference sequence snapshot
  - update method: explicit download to staging + checksum review

## Update Workflow
1. Stage candidate updates under `artifacts/assets-refresh/reference/`.
2. Diff old/new sequence headers and lengths.
3. Recompute package checksums.
4. Update provenance notes and commit with rationale.

## Safety Diff Rules
- Always diff by checksums and sequence statistics.
- Do not replace references silently.
- Any sequence-content change requires review notes in commit message.
