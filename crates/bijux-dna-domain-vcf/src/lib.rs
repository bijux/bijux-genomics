//! VCF domain primitives: stage IDs, typed params, metrics, and registry materialization.

pub mod artifacts;
pub mod contracts;
pub mod coverage;
pub mod metrics;
pub mod params;
pub mod registry_emit;
pub mod run;
pub mod stage_baseline;
pub mod taxonomy;

pub use artifacts::{
    build_vcf_scientific_drift_report, VcfScientificDriftArtifactDeltaV1,
    VcfScientificDriftChangeKind, VcfScientificDriftMetricDeltaV1,
    VcfScientificDriftReportV1, VcfScientificDriftSnapshotV1,
    VCF_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION,
};
pub use metrics::{VcfCallSummaryMetricsV1, VcfFilterBreakdownMetricsV1, VcfStatsMetricsV1};
pub use registry_emit::{param_registry_toml, required_tools_toml};
pub use run::{
    required_vcf_bench_corpus_scenarios, vcf_bench_corpus_datasets, vcf_bench_corpus_manifest,
    VcfBenchCorpusDatasetManifestEntryV1, VcfBenchCorpusId, VcfBenchCorpusManifestV1,
    VcfBenchDataset, VcfBenchScenario, VCF_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION,
};
pub use stage_baseline::{
    VcfInvariantsPreset, VcfStage, STAGE_CALL, STAGE_FILTER_READS, STAGE_PREFIX, STAGE_STATS,
};
pub use taxonomy::{
    validate_downstream_transition, CoverageRegime, DomainSupportStatus, VcfDomainStage,
    VcfStageKind, VCF_FORBIDDEN_TRANSITIONS, VCF_STAGE_ORDER_DOWNSTREAM, VCF_STAGE_TAXONOMY,
};

pub const VCF_STAGE_ID_CATALOG: &[&str] = &[
    "vcf.admixture",
    "vcf.call",
    "vcf.call_diploid",
    "vcf.call_gl",
    "vcf.call_pseudohaploid",
    "vcf.damage_filter",
    "vcf.demography",
    "vcf.filter",
    "vcf.gl_propagation",
    "vcf.ibd",
    "vcf.imputation",
    "vcf.impute",
    "vcf.pca",
    "vcf.phasing",
    "vcf.population_structure",
    "vcf.postprocess",
    "vcf.prepare_reference_panel",
    "vcf.qc",
    "vcf.roh",
    "vcf.stats",
];
pub const VCF_PARAMS_CATALOG: &[&str] = &[
    "bijux.vcf.call.params",
    "bijux.vcf.filter.params",
    "bijux.vcf.stats.params",
    "bijux.vcf.call_gl.params",
    "bijux.vcf.call_diploid.params",
    "bijux.vcf.call_pseudohaploid.params",
    "bijux.vcf.damage_filter.params",
    "bijux.vcf.gl_propagation.params",
];
pub const VCF_METRICS_CATALOG: &[&str] =
    &["bijux.vcf.call_summary.v1", "bijux.vcf.filter_breakdown.v1", "bijux.vcf.stats.v1"];
pub const VCF_PRODUCTION_TOOLS: &[&str] = &["bcftools"];
