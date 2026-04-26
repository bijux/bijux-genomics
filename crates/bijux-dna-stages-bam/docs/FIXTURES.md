# FIXTURES

## Why fixtures exist
Fixtures are minimal but representative samples used to validate observer parsing and contract stability.

## Fixture inventory
- `tests/fixtures/observer/default/flagstat.txt` — validates samtools flagstat parsing.
- `tests/fixtures/observer/default/idxstats.txt` — validates samtools idxstats parsing.
- `tests/fixtures/observer/default/samtools.depth.txt` — validates samtools depth parsing.
- `tests/fixtures/observer/default/stats.txt` — validates samtools stats parsing.
- `tests/fixtures/observer/default/mosdepth.summary.txt` — validates mosdepth summary parsing.
- `tests/fixtures/observer/default/gc_bias.metrics.txt` — validates Picard GC-bias parsing.
- `tests/fixtures/observer/default/insert_size.metrics.txt` — validates Picard insert-size parsing.
- `tests/fixtures/observer/default/mapdamage2.txt` — validates mapDamage2 parsing.
- `tests/fixtures/observer/default/damageprofiler.json` — validates damageprofiler JSON parsing.
- `tests/fixtures/observer/default/pydamage.json` — validates pydamage parsing.
- `tests/fixtures/observer/default/contamination.json` — validates contamination parser.
- `tests/fixtures/observer/default/sex.json` — validates sex inference parser.
- `tests/fixtures/observer/default/preseq.txt` — validates preseq parsing.
- `tests/fixtures/observer_snapshot/default/observer_snapshot.json` — canonical aggregate observer snapshot.
- `tests/fixtures/observer_snapshots/default/*.json` — per-tool canonical observer snapshots.
- `tests/fixtures/stage_contracts/default/stage_contracts.json` — canonical stage contract snapshot.
