# bijux-dna-domain-vcf Public API

`src/lib.rs` is the public facade. Consumers should use these exports instead of reaching into
private implementation details.

## Public Modules

- `artifacts`
- `contracts`
- `coverage`
- `metrics`
- `params`
- `parsers`
- `registry_emit`
- `run`
- `stage_baseline`
- `taxonomy`

## Major Export Groups

- Stage catalogs: `VCF_STAGE_ID_CATALOG`, `VCF_PARAMS_CATALOG`, `VCF_METRICS_CATALOG`, and
  `VCF_PRODUCTION_TOOLS`.
- Baseline stage IDs: `VcfStage`, `STAGE_CALL`, `STAGE_FILTER_READS`, `STAGE_STATS`, and
  `STAGE_PREFIX`.
- Downstream taxonomy: `VcfDomainStage`, `VcfStageKind`, `CoverageRegime`,
  `DomainSupportStatus`, `VCF_STAGE_TAXONOMY`, `VCF_STAGE_ORDER_DOWNSTREAM`, and
  `VCF_FORBIDDEN_TRANSITIONS`.
- Metrics: `VcfCallSummaryMetricsV1`, `VcfFilterBreakdownMetricsV1`, and `VcfStatsMetricsV1`.
- Parser normalization: `parse_bcftools_stage_metrics`, `parse_angsd_stage_metrics`,
  `parse_eigensoft_stage_metrics`, `parse_imputation_stage_metrics`,
  `parse_phasing_stage_metrics`, `parse_plink_stage_metrics`,
  `parse_plink2_stage_metrics`, and `parse_segment_stage_metrics`.
- Registry rendering: `param_registry_toml` and `required_tools_toml`.
- Corpus manifests: `VcfBenchCorpusId`, `VcfBenchScenario`, `VcfBenchCorpusManifestV1`,
  `vcf_bench_corpus_manifest`, and `required_vcf_bench_corpus_scenarios`.
- Scientific drift: `VcfScientificDriftSnapshotV1`, `VcfScientificDriftReportV1`, and
  `build_vcf_scientific_drift_report`.
- Workflow contracts: validation, artifact-class, reference-context, filter-evidence,
  normalization, calling-mode, stats-report, panel-boundary, and population-guardrail exports
  from `contracts`.

## Stability Rules

- New public modules and catalogs need docs and contract tests.
- Generated registry output changes must update the committed config artifact in the same change.
- Internal helper modules should stay private unless a downstream consumer needs a stable domain
  contract.

## Strict mini-fixture genotype observations

`summarize_fixture_genotypes(VcfFixtureGenotypeInput)` accepts caller-acquired
text and a diagnostic source path, returning `VcfFixtureGenotypeSummary` with
`VcfFixtureSampleGenotypes` and `VcfFixtureAlleleFrequency` observations. These
Rust rows have no persisted schema. The operation does not acquire the path or
publish expected-truth JSON. Sample/record order, duplicate sample aggregation,
last-seen site frequencies and existing strict fixture errors are preserved.
Malformed allele tokens and partial numeric calls such as `0/.` are refused;
recognized missing encodings remain missing observations. The broader aggregate
genotype summary remains a distinct contract.
