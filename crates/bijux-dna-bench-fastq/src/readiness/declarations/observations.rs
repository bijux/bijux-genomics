/// A serving binding before output-contract assessment. Preserve order and multiplicity.
#[derive(Debug, Clone)]
pub struct ServingBinding {
    pub stage_id: String,
    pub tool_id: String,
    pub adapter_status: String,
}
/// Canonical planner declaration fields consumed by benchmark output assessment.
#[derive(Debug, Clone)]
pub struct OutputDeclaration {
    pub declared_output_ids: Vec<String>,
    pub execution_expected_output_ids: Vec<String>,
    pub stage_expected_artifact_ids: Vec<String>,
}
/// Canonical planner assessment; the adapter supplies its actual completeness predicate.
#[derive(Debug, Clone)]
pub struct OutputAssessment {
    pub complete: bool,
    pub normalized_metrics_output_id: Option<String>,
    pub raw_output_artifact_ids: Vec<String>,
    pub missing_declarations: Vec<String>,
}
impl OutputAssessment {
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
}
/// Result binding retained until owner domain selection and full identity admission.
#[derive(Debug, Clone)]
pub struct ExpectedResultBinding {
    pub domain: String,
    pub result_row_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_family_id: String,
    pub fixture_id: String,
    pub sample_scope: String,
}
/// Inventory execution capability, mapped exhaustively from canonical declarations.
#[derive(Debug, Clone, Copy)]
pub enum InventoryReadinessKind {
    DryRun,
    Smoke,
    DryOrSmoke,
}
/// A declared stage and its execution capability.
#[derive(Debug, Clone)]
pub struct InventoryStage {
    pub stage_id: String,
    pub readiness_kind: InventoryReadinessKind,
}
/// Canonical inventory order is retained until owner admission.
#[derive(Debug, Clone)]
pub struct StageInventory {
    pub stages: Vec<InventoryStage>,
}
/// Benchmark support admission without catalog-orchestration dependencies.
#[derive(Debug, Clone)]
pub struct StageAdmission {
    pub tool_id: String,
    pub support_status: String,
}
/// Canonical argv and shell command are independent representations, never split or rebuilt.
#[derive(Debug, Clone)]
pub struct RenderedCommandObservation {
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_kind: String,
    pub argv: Vec<String>,
    pub command: String,
}
