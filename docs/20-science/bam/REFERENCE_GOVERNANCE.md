# BAM Reference Governance

## What
Defines which BAM stages are bound to governed reference banks before tool execution or scientific interpretation.

## Why
Reference-linked BAM stages are the easiest place to create silent scientific drift: wrong contig naming, wrong panel/reference bundle, or a contamination database that does not match the operator claim. This file keeps those dependencies explicit.

## Non-goals
- Replacing the lower-level reference authority implementation.
- Listing every artifact emitted by reference materialization.
- Claiming that stages without `bank_hooks` are free from all biological assumptions.

## Contracts
- Any BAM stage with non-empty `bank_hooks` in
  [domain/bam/stages/](../../../domain/bam/stages/) must appear exactly once here.
- Stages without `bank_hooks` are intentionally omitted; this file governs reference-bound stages only.
- The banks listed here are refusal boundaries, not best-effort hints.
- Pinned default reference-owning stages stay documented in
  [../../../domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).

| Stage | Required banks | Why it exists |
| --- | --- | --- |
| bam.align | reference_bank | Alignment reports are only interpretable when the governed reference bundle and naming scheme are explicit. |
| bam.mapping_summary | reference_bank | Mapping summaries inherit the same reference identity and naming assumptions as alignment. |
| bam.endogenous_content | reference_bank | Endogenous-content estimates are only meaningful against a governed reference target. |
| bam.gc_bias | reference_bank | GC-bias summaries require the governed reference GC context, not an incidental local index. |
| bam.contamination | reference_bank, contamination_db_bank | Contamination inference depends both on the alignment reference bundle and on the governed contamination database/model inputs. |

## Runtime Rules
- Reference and index inputs must come from lock-backed materialization under the governed reference authority surface.
- Contig/build mismatches are refusal conditions, not warnings.
- Reference-bound runs must emit a reference manifest with bank identity and checksum provenance.

## Failure modes
- Using the right tool on the wrong reference bank produces scientifically wrong results that still look operationally valid.
- Hiding contamination database dependencies makes contamination comparisons non-auditable across runs.
