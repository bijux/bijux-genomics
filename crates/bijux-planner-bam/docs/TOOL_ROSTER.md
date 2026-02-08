# TOOL_ROSTER

## Registry
Tool adapters live under:
- `src/tool_adapters/tools/catalog.rs`
- `src/tool_adapters/stages_*.rs`

## Tools and rationale
### Pre phase
- `bwa` — primary aligner for modern reads.
- `bowtie2` — alternative aligner for compatibility.
- `samtools` — validation + QC utilities.

### Core phase
- `samtools` — sort/index/coverage utilities.
- `gatk` — markdup alternative.
- `picard` — markdup implementation.
- `mosdepth` — coverage summary.

### Downstream phase
- `mapDamage2` — damage profiling.
- `pydamage` — contamination/authenticity modeling.
- `rxy` — sex inference.

## Selection code
Selection is implemented in `src/selection/tool_selection.rs` and wired to stage adapters in
`src/tool_adapters/bam.rs`.
