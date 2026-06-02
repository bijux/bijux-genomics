# STAGE_CONTRACTS

This document is strictly maintained. Update it alongside
`tests/contracts/contract_snapshots.rs`, observer fixtures, and parser snapshots.
Manual drift is treated as a contract break.

## Stage Registry

This list mirrors the `BamStage::all()` registry consumed by this crate. Keep it
in sync with `bijux-dna-domain-bam` when adding, renaming, or removing stages.

| Stage | Phase | Primary tools | Audit focus |
| --- | --- | --- | --- |
| `bam.align` | pre | bwa, bowtie2 | aligned BAM, index, flagstat, idxstats, samtools stats |
| `bam.validate` | pre | samtools | validation report, flagstat |
| `bam.qc_pre` | pre | samtools | flagstat, idxstats, samtools stats |
| `bam.mapping_summary` | pre | samtools, picard | flagstat, idxstats, mapping stats, mapping summary |
| `bam.filter` | core | samtools, bamtools, bedtools | filtered BAM, index, before/after flagstat and idxstats, filter summary |
| `bam.mapq_filter` | core | samtools, bamtools | filtered BAM, index, before/after flagstat and idxstats, MAPQ-filter summary |
| `bam.length_filter` | core | samtools, picard | filtered BAM, index, before/after flagstat and idxstats, length-filter summary |
| `bam.markdup` | core | picard, samtools | duplicate-marked BAM, index, before/after flagstat and idxstats, markdup summary |
| `bam.duplication_metrics` | core | samtools, picard | duplication report, histogram, and duplication summary |
| `bam.complexity` | core | preseq | complexity report, complexity curve, and saturation summary |
| `bam.coverage` | core | mosdepth, samtools, bedtools | coverage summary and depth sidecar |
| `bam.insert_size` | core | picard | insert-size metrics and histogram |
| `bam.gc_bias` | core | picard | GC-bias metrics and plot |
| `bam.endogenous_content` | core | samtools | endogenous-content report |
| `bam.overlap_correction` | core | bamutil | overlap-corrected BAM and index |
| `bam.damage` | downstream | pydamage, mapdamage2 | DNA damage reports |
| `bam.authenticity` | downstream | authenticct | authenticity report |
| `bam.contamination` | downstream | angsd | contamination report |
| `bam.sex` | downstream | rxy | sex inference report |
| `bam.bias_mitigation` | downstream | mapdamage2 | bias report |
| `bam.recalibration` | downstream | gatk | recalibrated BAM, index, recalibration report |
| `bam.haplogroups` | downstream | yleaf | haplogroup report |
| `bam.genotyping` | downstream | angsd, bcftools | genotyping report |
| `bam.kinship` | downstream | king | kinship report |

## Observer Contracts

- Parser outputs must serialize to canonical JSON deterministically.
- Unknown fields in supported tool outputs are ignored unless the BAM domain
  parser documents strict handling for that format.
- Missing required fields must fail with a parser error that identifies the tool
  or field closely enough for fixture debugging.
- Metric discovery must prefer stage-specific output names before generic names.

## Supported Observer Outputs

- `flagstat.txt`, `flagstat.after.txt`, `filter.flagstat.txt`, `markdup.flagstat.txt`
- `idxstats.txt`, `idxstats.after.txt`
- `samtools_stats.txt`, `stats.txt`
- `coverage.mosdepth.summary.txt`, `mosdepth.summary.txt`
- `coverage.depth.txt`, `depth.txt`, `samtools.depth.txt`
- `preseq.txt`
- `insert_size.metrics.txt`
- `gc_bias.metrics.txt`
- `damage.pydamage.json`, `pydamage.json`
- `damage.mapdamage2.txt`, `mapdamage2.txt`
- `damage.profiler.json`, `damageprofiler.json`
- `contamination.json`
- `sex.json`
- `CASE.json`

## Fixture Inventory

- `tests/fixtures/observer/default/*`: representative parser inputs.
- `tests/fixtures/observer_snapshot/default/observer_snapshot.json`: canonical aggregate observer
  snapshot.
- `tests/fixtures/observer_snapshots/default/*.json`: per-tool canonical observer snapshots.
- `tests/fixtures/stage_contracts/default/stage_contracts.json`: canonical stage contract snapshot.

## Canonical examples

### bam.damage

`damage.pydamage.json`

```json
{"reference":"ref.fa","damage":{"5p_C_to_T":0.12,"3p_G_to_A":0.10}}
```

`damage.mapdamage2.txt`
```text
Chr	Pos	5pC>T	3pG>A
chr1	1	0.12	0.10
```

`stage.metrics.json`
```json
{"schema_version":"bijux.stage.metrics.v1","stage_id":"bam.damage","tool_id":"pydamage","runtime_s":1.2,"wall_time_ms":1200,"memory_mb":256.0,"exit_code":0}
```

### bam.mapping_summary
- Required artifacts:
`flagstat.txt`, `idxstats.txt`, one governed `stats` artifact (`samtools_stats.txt` or `alignment_summary.metrics.txt`), `mapping.summary.json`, `stage.metrics.json`

### bam.mapq_filter
- Required artifacts:
`filtered.bam`, `filtered.bam.bai`, `flagstat.before.txt`, `flagstat.after.txt`, `idxstats.before.txt`, `idxstats.after.txt`, `mapq_filter.summary.json`, `stage.metrics.json`

### bam.filter
- Required artifacts:
`filtered.bam`, `filtered.bam.bai`, `flagstat.before.txt`, `flagstat.after.txt`, `idxstats.before.txt`, `idxstats.after.txt`, `filter.summary.json`, `stage.metrics.json`

### bam.length_filter
- Required artifacts:
`filtered.bam`, `filtered.bam.bai`, `flagstat.before.txt`, `flagstat.after.txt`, `idxstats.before.txt`, `idxstats.after.txt`, `length_filter.summary.json`, `stage.metrics.json`

### bam.markdup
- Required artifacts:
`markdup.bam`, `markdup.bam.bai`, `flagstat.before.txt`, `flagstat.after.txt`, `idxstats.before.txt`, `idxstats.after.txt`, `markdup.summary.json`, `stage.metrics.json`

### bam.duplication_metrics
- Required artifacts:
`duplication.metrics.json`, `duplication.histogram.txt`, `duplication.summary.json`, `stage.metrics.json`

### bam.complexity
- Required artifacts:
`complexity.json`, `complexity_curve.tsv`, `complexity.summary.json`, `stage.metrics.json`

### bam.coverage
- Required artifacts:
`coverage.mosdepth.summary.txt`, `coverage.depth.txt`, `stage.metrics.json`

### bam.insert_size
- Required artifacts:
`insert_size.metrics.txt`, `insert_size.histogram.pdf`, `insert_size.summary.json`, `stage.metrics.json`

### bam.gc_bias
- Required artifacts:
`gc_bias.metrics.txt`, `gc_bias.plot.pdf`, `gc_bias.summary.json`, `stage.metrics.json`
- Local smoke benchmark row:
`gc_bias.tsv` with governed GC-bin, normalized coverage, window count, and read-start count columns

## References

- mapDamage2: Jonsson et al. 2013.
- preseq: Daley and Smith 2013.
- mosdepth: Pedersen and Quinlan 2018.
