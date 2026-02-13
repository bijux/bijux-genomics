# VCF Fixture Format

Each fixture file under `domain/vcf/fixtures/<stage>/*.txt` must define:
- `tool=<tool_id>`
- `tool_version=<pinned|semver|digest>`
- `stage=<domain.stage>`
- `domain=vcf`
- `fixture_kind=<truth|smoke|negative>`
- `command=<tool invocation entrypoint>`
- `args=<cli args or empty>`
- `expected_outputs=<artifact ids or token>`
- `expected_stdout_patterns=<token list or placeholder>`

Stage coverage expectations:
- `vcf.population_structure/*`: at least one fixture producing `population_structure_report`.
- `vcf.roh/*`: at least one fixture producing `roh_report`.
- `vcf.ibd/*`: at least one fixture producing `ibd_segments`.
- `vcf.demography/*`: at least one fixture producing `demography_report`.
