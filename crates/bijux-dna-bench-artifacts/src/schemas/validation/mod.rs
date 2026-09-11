//! Validate complete governed schema files and retain diagnostic reports.
mod aggregate;
mod contracts;
mod io;
mod model;
mod service;
mod shared;
mod vcf;
pub use aggregate::validate_all_domain_schemas;
pub use contracts::{
    DEFAULT_ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_PATH, DEFAULT_BAM_SCHEMA_VALIDATION_REPORT_PATH,
    DEFAULT_FASTQ_SCHEMA_VALIDATION_REPORT_PATH, DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH,
};
pub use model::{
    AllDomainSchemaValidationDomainRow, AllDomainSchemaValidationReport, BamSchemaValidationReport,
    FastqSchemaValidationReport, SchemaDomain, SharedSchemaValidationRow,
    VcfSchemaValidationReport, VcfSchemaValidationRow,
};
pub use service::{validate_bam_schemas, validate_fastq_schemas, validate_vcf_schemas};
