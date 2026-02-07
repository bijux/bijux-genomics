# STAGE_LIST

## Essentials
- bam.sort — sort alignments (inputs: BAM, outputs: sorted BAM, metrics: sort time)
- bam.index — index BAM (inputs: BAM, outputs: BAI, metrics: index stats)

## Recommended
- bam.markdup — mark duplicates (inputs: BAM, outputs: dedup BAM, metrics: dup rate)

## Optional
- bam.damage — damage profiling (inputs: BAM, outputs: report, metrics: damage curves)
- bam.contamination — contamination estimates (inputs: BAM, outputs: report, metrics: contaminant rate)
