# FIXTURES

## Why fixtures exist
Fixtures are minimal but representative samples used to validate observer parsing and contract stability.

## Fixture inventory
- `tests/fixtures/observer/flagstat.txt` — validates samtools flagstat parsing.
- `tests/fixtures/observer/idxstats.txt` — validates samtools idxstats parsing.
- `tests/fixtures/observer/samtools.depth.txt` — validates samtools depth parsing.
- `tests/fixtures/observer/stats.txt` — validates samtools stats parsing.
- `tests/fixtures/observer/mosdepth.summary.txt` — validates mosdepth summary parsing.
- `tests/fixtures/observer/mapdamage2.txt` — validates mapDamage2 parsing.
- `tests/fixtures/observer/damageprofiler.json` — validates damageprofiler JSON parsing.
- `tests/fixtures/observer/pydamage.json` — validates pydamage parsing.
- `tests/fixtures/observer/contamination.json` — validates contamination parser.
- `tests/fixtures/observer/sex.json` — validates sex inference parser.
- `tests/fixtures/observer/preseq.txt` — validates preseq parsing.
- `tests/fixtures/observer_snapshot.json` — canonical observer snapshot contract.
- `tests/fixtures/observer_snapshots/*.json` — per-tool canonical observer snapshots.
- `tests/fixtures/stage_contracts.json` — canonical stage contract snapshot.
