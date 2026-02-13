# VCF Fixture Format

Each fixture file under `domain/vcf/fixtures/<stage>/*.txt` must define:
- `tool=<tool_id>`
- `stage=<domain.stage>`
- `domain=vcf`
- `fixture_kind=<truth|smoke|negative>`
- `args=<cli args or empty>`
- `expected_outputs=<artifact ids or token>`
