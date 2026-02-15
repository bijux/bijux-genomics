# STAGE_CONTRACTS

This document is strictly maintained. Update it alongside `tests/contracts/contract_snapshots.rs`.
Manual drift is treated as a contract break.

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
`flagstat.txt`, `idxstats.txt`, `samtools_stats.txt`, `mapping.summary.json`, `stage.metrics.json`

### bam.mapq_filter
- Required artifacts:
`filtered.bam`, `filtered.bam.bai`, `flagstat.before.txt`, `flagstat.after.txt`, `mapq_filter.summary.json`, `stage.metrics.json`

### bam.length_filter
- Required artifacts:
`filtered.bam`, `filtered.bam.bai`, `length_filter.summary.json`, `stage.metrics.json`
