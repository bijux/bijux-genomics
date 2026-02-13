# BAM Fixture Format

Each fixture file under `domain/bam/fixtures/<stage>/*.txt` must define:
- `tool=<tool_id>`
- `tool_version=<pinned|semver|digest>`
- `stage=<domain.stage>`
- `domain=bam`
- `fixture_kind=<truth|smoke|negative>`
- `command=<tool invocation entrypoint>`
- `args=<cli args or empty>`
- `expected_outputs=<artifact ids or token>`
- `expected_stdout_patterns=<token list or placeholder>`
