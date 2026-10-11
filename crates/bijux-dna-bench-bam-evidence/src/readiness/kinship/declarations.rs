#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::BAM_KINSHIP;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
pub(in crate::readiness) const BAM_KINSHIP_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_kinship_ready.v1";
pub(in crate::readiness) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(in crate::readiness) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(in crate::readiness) const CHECKED_SURFACE_COUNT: usize = 8;
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const LOCAL_PROOF_KIND_SMOKE: &str = "local_smoke";
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const LOCAL_PROOF_SAMPLE_ID: &str = "sample-set";
pub(in crate::readiness) const EXPECTED_SCHEMA_REQUIRED_KEYS: [&str; 9] = [
    "schema_version",
    "stage_id",
    "tool_id",
    "tool_version",
    "execution",
    "outputs_count",
    "artifacts",
    "contracts",
    "normalized_keys",
];
#[derive(Debug, Clone, Copy)]
pub(in crate::readiness) struct BamBindingSpec {
    pub(in crate::readiness) stage_id: &'static str,
    pub(in crate::readiness) tool_id: &'static str,
    pub(in crate::readiness) fixture_id: &'static str,
    pub(in crate::readiness) required_output_ids: &'static [&'static str],
    pub(in crate::readiness) normalized_metrics_output_id: &'static str,
    pub(in crate::readiness) schema_extension_id: &'static str,
    pub(in crate::readiness) expected_local_proof_kind: &'static str,
    pub(in crate::readiness) expected_local_proof_tool_id: &'static str,
    pub(in crate::readiness) required_local_proof_fields: &'static [&'static str],
    pub(in crate::readiness) required_local_proof_output_ids: &'static [&'static str],
    pub(in crate::readiness) report_section_id: &'static str,
    pub(in crate::readiness) summary_table_id: &'static str,
    pub(in crate::readiness) report_anchor_tool_id: &'static str,
}
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const KINSHIP_REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["kinship_report", "summary", "stage_metrics"];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const KINSHIP_REQUIRED_LOCAL_PROOF_FIELDS: [&str; 12] = [
    "cases",
    "pairwise_results",
    "sample_a",
    "sample_b",
    "overlap_snps",
    "kinship_coefficient",
    "relationship_label",
    "status",
    "observed_max_overlap_snps",
    "insufficiency_reason",
    "reference_panel",
    "population_scope",
];
pub(in crate::readiness) fn bam_binding_specs() -> Vec<BamBindingSpec> {
    #[cfg(feature = "bam_downstream")]
    {
        vec![BamBindingSpec {
            stage_id: BAM_KINSHIP,
            tool_id: "king",
            fixture_id: "corpus-01-kinship-mini",
            required_output_ids: &KINSHIP_REQUIRED_OUTPUT_IDS,
            normalized_metrics_output_id: "kinship_report",
            schema_extension_id: "bam_kinship_normalized_v1",
            expected_local_proof_kind: LOCAL_PROOF_KIND_SMOKE,
            expected_local_proof_tool_id: "king",
            required_local_proof_fields: &KINSHIP_REQUIRED_LOCAL_PROOF_FIELDS,
            required_local_proof_output_ids: &KINSHIP_REQUIRED_OUTPUT_IDS,
            report_section_id: "sample_identity",
            summary_table_id: "identity_inference",
            report_anchor_tool_id: "king",
        }]
    }
    #[cfg(not(feature = "bam_downstream"))]
    {
        Vec::new()
    }
}

pub(in crate::readiness) fn expected_stage_ids() -> BTreeSet<&'static str> {
    bam_binding_specs().into_iter().map(|spec| spec.stage_id).collect()
}
pub(in crate::readiness) fn expected_tool_ids_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, Vec<String>>::new();
    for spec in bam_binding_specs() {
        by_stage.entry(spec.stage_id.to_string()).or_default().push(spec.tool_id.to_string());
    }
    by_stage
}
pub(in crate::readiness) fn required_output_ids_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for spec in bam_binding_specs() {
        let stage_outputs = by_stage.entry(spec.stage_id.to_string()).or_default();
        for output_id in spec.required_output_ids {
            stage_outputs.insert((*output_id).to_string());
        }
    }
    by_stage
        .into_iter()
        .map(|(stage_id, output_ids)| (stage_id, output_ids.into_iter().collect()))
        .collect()
}
pub(in crate::readiness) fn expected_local_proof_kind_by_stage() -> BTreeMap<String, String> {
    let mut by_stage = BTreeMap::<String, String>::new();
    for spec in bam_binding_specs() {
        by_stage
            .entry(spec.stage_id.to_string())
            .or_insert_with(|| spec.expected_local_proof_kind.to_string());
    }
    by_stage
}
pub(in crate::readiness) fn required_local_proof_fields_by_stage() -> BTreeMap<String, Vec<String>>
{
    let mut by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for spec in bam_binding_specs() {
        let stage_fields = by_stage.entry(spec.stage_id.to_string()).or_default();
        for field in spec.required_local_proof_fields {
            stage_fields.insert((*field).to_string());
        }
    }
    by_stage
        .into_iter()
        .map(|(stage_id, fields)| (stage_id, fields.into_iter().collect()))
        .collect()
}
