/// Consumed compatibility declaration fields, before semantic admission.
#[derive(Debug, Clone)]
pub struct CorpusDeclaration {
    pub stage_id: String,
    pub compatibility_kind: String,
    pub fixture_id: Option<String>,
    pub benchmark_scope_id: Option<String>,
}
/// Canonical inventory cardinality and stage identities.
#[derive(Debug, Clone)]
pub struct StageInventory {
    pub stage_count: usize,
    pub stages: Vec<String>,
}
/// Capability facts and canonical readiness label observed from the FASTQ domain.
// Canonical capability flags describe independent facts, including incomplete declarations.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct FastqAssessment {
    pub readiness: &'static str,
    pub active: bool,
    pub runnable: bool,
    pub plannable: bool,
    pub declared: bool,
    pub comparable: bool,
    pub benchmark_normalized: bool,
    pub parse_normalized: bool,
}
/// Canonical BAM stage facts, observed without maintaining another stage registry.
#[derive(Debug, Clone)]
pub struct BamStageAssessment {
    pub tools: Vec<String>,
    pub has_parser_fixtures: bool,
    pub has_scientific_report_contract: bool,
    pub has_artifact_contract: bool,
}
/// Canonical planner labels, including missing-definition diagnostics.
#[derive(Debug, Clone)]
pub struct BamToolAssessment {
    pub support: &'static str,
    pub adapter: &'static str,
}
