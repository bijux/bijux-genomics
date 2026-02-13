# FASTQ Fixture Format

Each fixture file under `domain/fastq/fixtures/<stage>/*.txt` must define:
- `tool=<tool_id>`
- `stage=<domain.stage>`
- `domain=fastq`
- `fixture_kind=<truth|smoke|negative>`
- `args=<cli args or empty>`
- `expected_outputs=<artifact ids or token>`
