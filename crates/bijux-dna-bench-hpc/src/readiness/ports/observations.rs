//! Nonserialized projections of provider evidence used by local readiness policy.
#[derive(Debug, Clone)]
pub struct CorpusSamplesObservation {
    pub sample_count: usize,
    pub valid: bool,
}
#[derive(Debug, Clone)]
pub struct DamageFixtureObservation {
    pub expected_damage_path: String,
    pub valid: bool,
}
#[derive(Debug, Clone)]
pub struct EdnaFixtureObservation {
    pub manifest_path: String,
    pub sample_count: usize,
    pub expected_taxa_path: String,
    pub expected_taxa_count: usize,
    pub expected_taxa_output_row_count: usize,
    pub valid: bool,
}
#[derive(Debug, Clone)]
pub struct AmpliconFixtureObservation {
    pub manifest_path: String,
    pub sample_count: usize,
    pub control_count: usize,
    pub primer_table_row_count: usize,
    pub primers_tsv_path: String,
    pub expected_asv_row_count: usize,
    pub expected_asvs_path: String,
    pub chimera_expectation_row_count: usize,
    pub chimera_controls_fasta_path: String,
    pub chimera_expectations_path: String,
    pub valid: bool,
}
#[derive(Debug, Clone)]
pub struct TaxonomyFixtureObservation {
    pub valid: bool,
    pub taxa_count: usize,
    pub source_record_count: usize,
}
#[derive(Debug, Clone)]
pub struct ToolComparisonObservation {
    pub tsv_output_path: String,
    pub row_count: usize,
    pub observed_row_count: usize,
}
#[derive(Debug, Clone)]
pub struct CampaignPlanObservation {
    pub campaign_id: String,
    pub planned_job_count: usize,
}
#[derive(Debug, Clone)]
pub struct FoundationPreparationObservation {
    pub action_count: usize,
}
#[derive(Debug, Clone)]
pub struct SubmittedScriptObservation {
    pub script_path: String,
    pub code_path: String,
}
#[derive(Debug, Clone)]
pub struct MockSubmissionObservation {
    pub jobs: Vec<SubmittedScriptObservation>,
}
