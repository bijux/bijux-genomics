# BAM Fixture Format

Each fixture file under `domain/bam/fixtures/<stage>/*.txt` must define:
- `tool=<tool_id>`
- `stage=<domain.stage>`
- `domain=bam`
- `fixture_kind=<truth|smoke|negative>`
- `args=<cli args or empty>`
- `expected_outputs=<artifact ids or token>`
