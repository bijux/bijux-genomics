use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SharedSchemaValidationRow {
    pub stage_id: String,
    pub extension_id: String,
    pub required_key_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FastqSchemaValidationReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub shared_schema_path: String,
    pub passes_gate: bool,
    pub shared_schema_matches: bool,
    pub stage_count: usize,
    pub rows: Vec<SharedSchemaValidationRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamSchemaValidationReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub shared_schema_path: String,
    pub passes_gate: bool,
    pub shared_schema_matches: bool,
    pub stage_count: usize,
    pub rows: Vec<SharedSchemaValidationRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfSchemaValidationRow {
    pub stage_id: String,
    pub schema_version: String,
    pub schema_file: String,
    pub file_present: bool,
    pub exact_match: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfSchemaValidationReport {
    pub schema_version: &'static str,
    pub domain: &'static str,
    pub output_path: String,
    pub shared_schema_path: String,
    pub stage_dir: String,
    pub passes_gate: bool,
    pub shared_schema_matches: bool,
    pub stage_count: usize,
    pub required_stage_count: usize,
    pub exact_stage_schema_file_count: usize,
    pub missing_stage_schema_files: Vec<String>,
    pub unexpected_stage_schema_files: Vec<String>,
    pub rows: Vec<VcfSchemaValidationRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllDomainSchemaValidationDomainRow {
    pub domain: String,
    pub output_path: String,
    pub shared_schema_path: String,
    pub stage_dir: Option<String>,
    pub passes_gate: bool,
    pub shared_schema_matches: bool,
    pub stage_count: usize,
    pub required_stage_count: Option<usize>,
    pub exact_stage_schema_file_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllDomainSchemaValidationReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub schema_root: String,
    pub domain_count: usize,
    pub passed_domain_count: usize,
    pub failed_domain_count: usize,
    pub ok: bool,
    pub domains: Vec<AllDomainSchemaValidationDomainRow>,
}

/// Domain admitted by benchmark schema validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchemaDomain {
    Fastq,
    Bam,
    Vcf,
}
