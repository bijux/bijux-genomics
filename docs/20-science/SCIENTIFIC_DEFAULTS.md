# Scientific Defaults

This table documents scientifically-relevant defaults used by production and reference profiles.

| Domain | Default/Threshold | Meaning | Applies To | Rationale | Comparability Implication | References |
|---|---|---|---|---|---|---|
| FASTQ | `trim.min_len=25` (aDNA/reference) | Minimum retained read length after trimming | Paired-end aDNA shotgun reference | Preserves short endogenous fragments while excluding non-informative stubs | Runs are comparable only when this threshold and library model are unchanged | docs/20-science/fastq/GOLD_PIPELINE_SPEC.md |
| FASTQ | `trim.adapter_policy=ancient_strict` | Adapter detection/trimming policy | aDNA reference profiles | Ancient fragments are adapter-rich; strict policy reduces false retention | Adapter-retention metrics are not cross-comparable with relaxed adapter policies | docs/20-science/fastq/TOOLS_ROSTER.md |
| FASTQ | `trim.q_cutoff=20` | Quality trimming threshold | aDNA/reference FASTQ | Controls base-call noise while keeping ultra-short fragments | Base-retention and quality-shift comparisons require identical q cutoff | docs/20-science/fastq/STAGE_ASSUMPTIONS.md |
| FASTQ | `trim.polyx_policy=trim` | Poly-X tail trimming policy | aDNA/reference FASTQ | Removes sequencer/artifactual tails that bias downstream metrics | Length-shift and retention comparisons require same poly-X policy | docs/20-science/fastq/STAGE_ASSUMPTIONS.md |
| FASTQ | `merge.min_len=20` | Min merged read length | Paired-end aDNA reference | Supports overlap collapse for fragmented molecules | Merge-rate comparisons are invalid if min merged length differs | docs/20-science/fastq/GOLD_PIPELINE_SPEC.md |
| FASTQ | `merge.merge_overlap=11` | Minimum overlap for merge | Paired-end aDNA reference | Enforces biologically plausible pair collapse | Paired collapse metrics require fixed overlap threshold | docs/20-science/fastq/GOLD_PIPELINE_SPEC.md |
| BAM | `damage` stage required for aDNA | Estimate terminal deamination signal | aDNA BAM profiles | Damage signal is central for authenticity interpretation | Authenticity conclusions are not comparable without identical damage-stage inclusion | docs/20-science/VALIDITY_LIMITS.md |
| BAM | Damage expectation ranges by UDG | Expected terminal damage bands vary by treatment | aDNA BAM with UDG metadata | UDG chemistry changes apparent damage rates | UDG-mismatched runs must not be directly compared for damage magnitude | docs/20-science/VALIDITY_LIMITS.md |
| VCF | `vcf.stats` required | Variant summary, counts, Ti/Tv | VCF profiles | Baseline quality/provenance requirement for interpretation | Variant-level comparisons require same stats schema and reference build | docs/50-reference/PIPELINES.md |

## Library Model policy

All profile constructors must declare a `LibraryModel`:

- `layout`: `single_end` or `paired_end`
- `udg_treatment`: `none`, `partial`, `full`, `unknown`
- `platform_hint`: sequencing platform hint
- `assay_kind`: `shotgun`, `capture`, or `unknown`

No profile may rely on implicit library assumptions.

## Invariant severities

- `hard`: blocks production execution and marks profile invalid.
- `soft`: warning-level scientific risk; does not block by itself.

`invariants_report.json` is the canonical machine-readable report artifact.

## Purpose
This document defines the intended behavior and navigation contract for this topic.

## Scope
Applies only to the files and workflows referenced in this document.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.

## Examples
- Keeping `trim.min_len=25` unchanged preserves comparability for aDNA FASTQ retention metrics.
- Requiring `vcf.stats` across VCF profiles preserves baseline variant-quality interpretability.

## Failure modes
- Unreviewed threshold changes can invalidate cross-run scientific comparisons.
- Omitting required defaults from profile constructors introduces hidden scientific assumptions.
