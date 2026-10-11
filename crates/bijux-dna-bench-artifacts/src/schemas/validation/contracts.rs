pub const DEFAULT_FASTQ_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/fastq-schema-validation.json";
pub const DEFAULT_BAM_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/bam-schema-validation.json";
pub const DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-schema-validation.json";
pub const DEFAULT_ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domain-schema-validation.json";

pub(super) const FASTQ_SCHEMA_VALIDATION_REPORT_VERSION: &str =
    "bijux.bench.readiness.fastq_schema_validation.v1";
pub(super) const BAM_SCHEMA_VALIDATION_REPORT_VERSION: &str =
    "bijux.bench.readiness.bam_schema_validation.v1";
pub(super) const VCF_SCHEMA_VALIDATION_REPORT_VERSION: &str =
    "bijux.bench.readiness.vcf_schema_validation.v1";
pub(super) const ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_VERSION: &str =
    "bijux.bench.readiness.all_domain_schema_validation.v1";

pub(super) const REQUIRED_VCF_SCHEMA_STAGE_IDS: &[&str] = &[
    "vcf.call",
    "vcf.call_gl",
    "vcf.call_diploid",
    "vcf.call_pseudohaploid",
    "vcf.damage_filter",
    "vcf.demography",
    "vcf.gl_propagation",
    "vcf.qc",
    "vcf.pca",
    "vcf.population_structure",
    "vcf.admixture",
    "vcf.roh",
    "vcf.imputation_metrics",
    "vcf.ibd",
    "vcf.impute",
    "vcf.phasing",
    "vcf.postprocess",
    "vcf.prepare_reference_panel",
    "vcf.filter",
    "vcf.stats",
];
