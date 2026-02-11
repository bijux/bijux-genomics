# Scientific Defaults

This table documents scientifically-relevant defaults used by production and reference profiles.

| Domain | Default/Threshold | Meaning | Applies To | Rationale | References |
|---|---|---|---|---|---|
| FASTQ | `trim.min_len=25` (aDNA/reference) | Minimum retained read length after trimming | Paired-end aDNA shotgun | Preserves short endogenous fragments while excluding non-informative stubs | AdapterRemoval manual; aDNA preprocessing practice |
| FASTQ | `trim.adapter_policy=ancient_strict` | Adapter detection/trimming policy | aDNA profiles | Ancient fragments are adapter-rich; strict policy reduces false retention | AdapterRemoval / leeHom docs |
| FASTQ | `trim.q_cutoff=20` | Quality trimming threshold | aDNA/reference FASTQ | Controls base-call noise while keeping ultra-short fragments | fastp manual |
| FASTQ | `trim.polyx_policy=trim` | Poly-X tail trimming policy | aDNA/reference FASTQ | Removes sequencer/artifactual tails that bias downstream metrics | fastp manual |
| FASTQ | `merge.min_len=20` | Min merged read length | Paired-end aDNA | Supports overlap collapse for fragmented molecules | leeHom documentation |
| FASTQ | `merge.merge_overlap=11` | Minimum overlap for merge | Paired-end aDNA | Enforces biologically plausible pair collapse | leeHom documentation |
| BAM | `damage` stage required for aDNA | Estimate terminal deamination signal | aDNA BAM profiles | Damage signal is central for authenticity interpretation | mapDamage / PyDamage docs |
| BAM | Damage expectation ranges by UDG | Expected terminal damage bands vary by treatment | aDNA BAM with UDG metadata | UDG chemistry changes apparent damage rates | mapDamage / aDNA methods literature |
| VCF | `vcf.stats` required | Variant summary, counts, Ti/Tv | VCF profiles | Baseline quality/provenance requirement for interpretation | bcftools stats docs |

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
