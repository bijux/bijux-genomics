# VCF Fixture Format

Each fixture file under domain/vcf/fixtures/STAGE_ID/*.txt must define:
- `tool=<tool_id>`
- `tool_version=<pinned|semver|digest>`
- `stage=<domain.stage>`
- `domain=vcf`
- `fixture_kind=<truth|smoke|negative>`
- `command=<tool invocation entrypoint>`
- `args=<cli args or empty>`
- `expected_outputs=<artifact ids or token>`
- `expected_stdout_patterns=<token list or placeholder>`

## Fixture Directories
- `vcf.admixture`: intent = stage-specific command contract coverage for `vcf.admixture`.
- `vcf.call`: intent = stage-specific command contract coverage for `vcf.call`.
- `vcf.call_diploid`: intent = stage-specific command contract coverage for `vcf.call_diploid`.
- `vcf.call_gl`: intent = stage-specific command contract coverage for `vcf.call_gl`.
- `vcf.call_pseudohaploid`: intent = stage-specific command contract coverage for `vcf.call_pseudohaploid`.
- `vcf.damage_filter`: intent = stage-specific command contract coverage for `vcf.damage_filter`.
- `vcf.demography`: intent = stage-specific command contract coverage for `vcf.demography`.
- `vcf.filter`: intent = stage-specific command contract coverage for `vcf.filter`.
- `vcf.gl_propagation`: intent = stage-specific command contract coverage for `vcf.gl_propagation`.
- `vcf.ibd`: intent = stage-specific command contract coverage for `vcf.ibd`.
- `vcf.imputation_metrics`: intent = stage-specific command contract coverage for `vcf.imputation_metrics`.
- `vcf.impute`: intent = stage-specific command contract coverage for `vcf.impute`.
- `vcf.pca`: intent = stage-specific command contract coverage for `vcf.pca`.
- `vcf.phasing`: intent = stage-specific command contract coverage for `vcf.phasing`.
- `vcf.population_structure`: intent = stage-specific command contract coverage for `vcf.population_structure`.
- `vcf.postprocess`: intent = stage-specific command contract coverage for `vcf.postprocess`.
- `vcf.prepare_reference_panel`: intent = stage-specific command contract coverage for `vcf.prepare_reference_panel`.
- `vcf.qc`: intent = stage-specific command contract coverage for `vcf.qc`.
- `vcf.roh`: intent = stage-specific command contract coverage for `vcf.roh`.
- `vcf.stats`: intent = stage-specific command contract coverage for `vcf.stats`.
