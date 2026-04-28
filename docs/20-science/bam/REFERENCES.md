# BAM References

## What
Governed reference and citation ledger for BAM-stage tools, metrics, and downstream interpretation surfaces.

## Why
The BAM domain is not reviewable if tool admission, stage claims, and citation closure are mixed together. This file keeps the admitted tool surface explicit while showing where paper closure still has not been completed.

## Non-goals
- Exhaustive archaeological-genomics literature review.
- Replacing tool manuals or domain manifests.
- Pretending that `citation: pending:tool-publication` means the science backlog is closed.

## Contracts
- Every governed BAM tool must map to the exact `stage_ids` declared in
  [domain/bam/tools/](../../../domain/bam/tools/).
- Conservative release-review blocker status for BAM tools now lives in
  [../../../science/docs/upstream/bam/BAM_PRODUCTION_CLOSURE_LEDGER.tsv](../../../science/docs/upstream/bam/BAM_PRODUCTION_CLOSURE_LEDGER.tsv).
- Paper-status closure for governed tool citations stays tracked in
  [../../../science/docs/upstream/papers/TOOL_PAPER_MAP.tsv](../../../science/docs/upstream/papers/TOOL_PAPER_MAP.tsv).
- Repository-style upstream software locators stay governed by
  [../../../science/docs/upstream/github-repos/README.md](../../../science/docs/upstream/github-repos/README.md).
- `Primary locator` may point to an upstream project page when no locally governed paper locator exists yet, but the backlog must stay explicit.
- Planned tools remain visible here when they are already in the BAM tool manifest so reference closure work can be tracked before runtime promotion.

## Alignment, Validation, and Filtering
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| bwa | `bam.align` | governed paper locator present; runtime closure remains separate | https://github.com/lh3/bwa |
| bowtie2 | `bam.align` | governed paper locator present; runtime closure remains separate | https://github.com/BenLangmead/bowtie2 |
| samtools | `bam.validate`, `bam.qc_pre`, `bam.mapping_summary`, `bam.filter`, `bam.mapq_filter`, `bam.length_filter`, `bam.markdup`, `bam.duplication_metrics`, `bam.coverage`, `bam.endogenous_content` | governed paper locator present; runtime closure remains separate | https://github.com/samtools/samtools |
| bedtools | `bam.validate`, `bam.filter` | governed paper locator present; runtime closure remains separate | https://github.com/arq5x/bedtools2 |
| bamtools | `bam.validate`, `bam.filter`, `bam.mapq_filter` | governed paper locator present; runtime closure remains separate | https://github.com/pezmaster31/bamtools |
| mosdepth | `bam.coverage` | governed paper locator present; runtime closure remains separate | https://github.com/brentp/mosdepth |
| picard | `bam.markdup`, `bam.length_filter`, `bam.duplication_metrics`, `bam.insert_size`, `bam.gc_bias` | governed software citation present; no standalone Picard paper is claimed | https://github.com/broadinstitute/picard |

## Damage, Authenticity, Contamination, and Inference
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| mapdamage2 | `bam.damage` | governed paper locator present; runtime closure remains separate | https://github.com/ginolhac/mapDamage |
| pydamage | `bam.damage` | governed paper locator present; runtime closure remains separate | https://github.com/maxibor/pydamage |
| damageprofiler | `bam.damage`, `bam.authenticity` | governed paper locator present; runtime closure remains separate | https://github.com/Integrative-Transcriptomics/DamageProfiler |
| pmdtools | `bam.damage`, `bam.authenticity` | governed method-paper citation present; runtime closure remains separate | https://github.com/pontussk/PMDtools |
| addeam | `bam.damage` | governed paper locator present; runtime closure remains separate | https://github.com/LouisPwr/AdDeam |
| authenticct | `bam.authenticity` | governed paper locator present; runtime closure remains separate | https://github.com/StephanePeyregne/AuthentiCT |
| schmutzi | `bam.contamination` | governed paper locator present; runtime closure remains separate | https://github.com/grenaud/schmutzi |
| verifybamid2 | `bam.contamination` | governed paper locator present; runtime closure remains separate | https://github.com/Griffan/VerifyBamID |
| contammix | `bam.contamination` | package/software locator captured; paper mapping still pending | https://bioconductor.org/packages/contamMix |
| rxy | `bam.sex` | governed tool contract still points to a local interim locator; external citation locator still needed | https://github.com/bijux/bijux-genomics |
| yleaf | `bam.sex`, `bam.haplogroups` | governed paper locator present; runtime closure remains separate | https://github.com/genid/Yleaf |
| angsd | `bam.sex`, `bam.kinship` | governed paper locator present; runtime closure remains separate | https://github.com/ANGSD/angsd |
| king | `bam.kinship` | governed paper locator present; runtime closure remains separate | https://www.kingrelatedness.com/ |

## Planned Expansion and Open Citation Backlog
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| preseq | `bam.complexity` | governed paper locator present; runtime closure remains separate | https://github.com/smithlabcode/preseq |
| bamutil | `bam.overlap_correction` | upstream software locator captured; paper mapping still pending | https://github.com/statgen/bamUtil |
| gatk | `bam.recalibration` | governed paper locator present; promotion and runtime evidence still pending | https://github.com/broadinstitute/gatk |
| ngsbriggs | `bam.damage` | governed paper locator and upstream repository are now present; promotion and runtime evidence still pending | https://github.com/RAHenriksen/ngsBriggs |

## Failure modes
- A tool listed against the wrong stage creates fake scientific support for a runtime boundary we do not actually govern.
- Replacing an explicit backlog with hand-wavy prose hides citation debt instead of making it reviewable.
- Interim upstream locators such as `rxy` must stay visible until they are repaired in the governed tool catalog and science evidence backlog.
