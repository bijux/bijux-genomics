# BAM Fixture Format

Each fixture file under domain/bam/fixtures/STAGE_ID/*.txt must define:
- `tool=<tool_id>`
- `tool_version=<pinned|semver|digest>`
- `stage=<domain.stage>`
- `domain=bam`
- `fixture_kind=<truth|smoke|negative>`
- `command=<tool invocation entrypoint>`
- `args=<cli args or empty>`
- `expected_outputs=<artifact ids or token>`
- `expected_stdout_patterns=<token list or placeholder>`

For `bam.validate` local-smoke coverage, the governed passing fixture is an explicit tiny binary
BAM with a sibling BAI and shared reference FASTA. The refusal case is an explicit malformed BAM
payload so deterministic contract checks exercise real BAM parsing rather than SAM-text proxies.

## Fixture Directories
- `bam.align`: intent = stage-specific command contract coverage for `bam.align`.
- `bam.authenticity`: intent = stage-specific command contract coverage for `bam.authenticity`.
- `bam.bias_mitigation`: intent = stage-specific command contract coverage for `bam.bias_mitigation`.
- `bam.complexity`: intent = stage-specific command contract coverage for `bam.complexity`.
- `bam.contamination`: intent = stage-specific command contract coverage for `bam.contamination`.
- `bam.coverage`: intent = stage-specific command contract coverage for `bam.coverage`.
- `bam.damage`: intent = stage-specific command contract coverage for `bam.damage`.
- `bam.duplication_metrics`: intent = stage-specific command contract coverage for `bam.duplication_metrics`.
- `bam.endogenous_content`: intent = stage-specific command contract coverage for `bam.endogenous_content`.
- `bam.filter`: intent = stage-specific command contract coverage for `bam.filter`.
- `bam.gc_bias`: intent = stage-specific command contract coverage for `bam.gc_bias`.
- `bam.genotyping`: intent = stage-specific command contract coverage for `bam.genotyping`.
- `bam.haplogroups`: intent = stage-specific command contract coverage for `bam.haplogroups`.
- `bam.insert_size`: intent = stage-specific command contract coverage for `bam.insert_size`.
- `bam.kinship`: intent = stage-specific command contract coverage for `bam.kinship`.
- `bam.length_filter`: intent = stage-specific command contract coverage for `bam.length_filter`.
- `bam.mapping_summary`: intent = stage-specific command contract coverage for `bam.mapping_summary`.
- `bam.mapq_filter`: intent = stage-specific command contract coverage for `bam.mapq_filter`.
- `bam.markdup`: intent = stage-specific command contract coverage for `bam.markdup`.
- `bam.overlap_correction`: intent = stage-specific command contract coverage for `bam.overlap_correction`.
- `bam.qc_pre`: intent = stage-specific command contract coverage for `bam.qc_pre`.
- `bam.recalibration`: intent = stage-specific command contract coverage for `bam.recalibration`.
- `bam.sex`: intent = stage-specific command contract coverage for `bam.sex`.
- `bam.validate`: intent = stage-specific command contract coverage for `bam.validate`.
