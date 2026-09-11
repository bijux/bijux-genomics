#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::BAM_HAPLOGROUPS;
use bijux_dna_core::id_catalog::{BAM_CONTAMINATION, BAM_SEX};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
pub(in crate::readiness) const BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_contamination_sex_haplogroups_ready.v1";
pub(in crate::readiness) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(in crate::readiness) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(in crate::readiness) const CHECKED_SURFACE_COUNT: usize = 8;
pub(in crate::readiness) const EXPECTED_FIXTURE_ID: &str = "corpus-01-adna-bam-mini";
pub(in crate::readiness) const LOCAL_PROOF_KIND_SMOKE: &str = "local_smoke";
pub(in crate::readiness) const LOCAL_PROOF_KIND_READY_PLAN: &str = "local_ready_plan";
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
    pub(in crate::readiness) required_output_ids: &'static [&'static str],
    pub(in crate::readiness) normalized_metrics_output_id: &'static str,
    pub(in crate::readiness) schema_extension_id: &'static str,
    pub(in crate::readiness) expected_local_proof_kind: &'static str,
    pub(in crate::readiness) expected_local_proof_tool_id: &'static str,
    pub(in crate::readiness) required_local_proof_fields: &'static [&'static str],
    pub(in crate::readiness) required_local_proof_output_ids: &'static [&'static str],
    pub(in crate::readiness) required_local_proof_command_fragments: &'static [&'static str],
    pub(in crate::readiness) report_section_id: &'static str,
    pub(in crate::readiness) summary_table_id: &'static str,
    pub(in crate::readiness) report_anchor_tool_id: &'static str,
}
pub(in crate::readiness) const CONTAMINATION_REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["contamination_report", "summary", "stage_metrics"];
pub(in crate::readiness) const SEX_REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["sex_report", "summary", "stage_metrics"];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const HAPLOGROUPS_REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["haplogroups", "summary", "stage_metrics"];
pub(in crate::readiness) const CONTAMINATION_REQUIRED_LOCAL_PROOF_FIELDS: [&str; 4] =
    ["sample_id", "scope", "reference_panels", "emit_confidence_caveats"];
pub(in crate::readiness) const SEX_REQUIRED_LOCAL_PROOF_FIELDS: [&str; 11] = [
    "method",
    "chromosome_system",
    "minimum_y_sites",
    "x_coverage",
    "y_coverage",
    "autosomal_coverage",
    "x_to_y_ratio",
    "call",
    "confidence",
    "status",
    "sex_report",
];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const HAPLOGROUPS_REQUIRED_LOCAL_PROOF_FIELDS: [&str; 4] =
    ["sample_id", "reference_panel_id", "coverage_gate", "reference_build"];
pub(in crate::readiness) const CONTAMINATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS: [&str; 3] =
    ["contamination_report", "summary", "stage_metrics"];
pub(in crate::readiness) const SEX_REQUIRED_LOCAL_PROOF_OUTPUT_IDS: [&str; 3] =
    ["sex_report", "sex_summary", "stage_metrics"];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const HAPLOGROUPS_REQUIRED_LOCAL_PROOF_OUTPUT_IDS: [&str; 3] =
    ["haplogroups", "summary", "stage_metrics"];
pub(in crate::readiness) const CONTAMINATION_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS: [&str; 4] = [
    "adna_contamination_panel_screen.sam.bai",
    "adna_bam_reference.fasta",
    "adna_contamination_panel.dat",
    "artifacts/benchmark-smoke/readiness/local-ready/bam.contamination/contamination.summary.json",
];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const HAPLOGROUPS_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS: [&str; 5] = [
    "adna_y_haplogroup_panel.sam",
    "adna_y_haplogroup_panel.sam.bai",
    "adna_y_haplogroup_panel.tsv",
    "--reference_genome hg38",
    "artifacts/benchmark-smoke/readiness/local-ready/bam.haplogroups/haplogroups",
];
pub(in crate::readiness) fn bam_binding_specs() -> Vec<BamBindingSpec> {
    #[allow(unused_mut)]
    let mut specs = vec![
        BamBindingSpec {
            stage_id: BAM_CONTAMINATION,
            tool_id: "contammix",
            required_output_ids: &CONTAMINATION_REQUIRED_OUTPUT_IDS,
            normalized_metrics_output_id: "contamination_report",
            schema_extension_id: "bam_contamination_normalized_v1",
            expected_local_proof_kind: LOCAL_PROOF_KIND_READY_PLAN,
            expected_local_proof_tool_id: "verifybamid2",
            required_local_proof_fields: &CONTAMINATION_REQUIRED_LOCAL_PROOF_FIELDS,
            required_local_proof_output_ids: &CONTAMINATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
            required_local_proof_command_fragments:
                &CONTAMINATION_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS,
            report_section_id: "ancient_signal",
            summary_table_id: "damage_authenticity",
            report_anchor_tool_id: "schmutzi",
        },
        BamBindingSpec {
            stage_id: BAM_CONTAMINATION,
            tool_id: "schmutzi",
            required_output_ids: &CONTAMINATION_REQUIRED_OUTPUT_IDS,
            normalized_metrics_output_id: "contamination_report",
            schema_extension_id: "bam_contamination_normalized_v1",
            expected_local_proof_kind: LOCAL_PROOF_KIND_READY_PLAN,
            expected_local_proof_tool_id: "verifybamid2",
            required_local_proof_fields: &CONTAMINATION_REQUIRED_LOCAL_PROOF_FIELDS,
            required_local_proof_output_ids: &CONTAMINATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
            required_local_proof_command_fragments:
                &CONTAMINATION_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS,
            report_section_id: "ancient_signal",
            summary_table_id: "damage_authenticity",
            report_anchor_tool_id: "schmutzi",
        },
        BamBindingSpec {
            stage_id: BAM_CONTAMINATION,
            tool_id: "verifybamid2",
            required_output_ids: &CONTAMINATION_REQUIRED_OUTPUT_IDS,
            normalized_metrics_output_id: "contamination_report",
            schema_extension_id: "bam_contamination_normalized_v1",
            expected_local_proof_kind: LOCAL_PROOF_KIND_READY_PLAN,
            expected_local_proof_tool_id: "verifybamid2",
            required_local_proof_fields: &CONTAMINATION_REQUIRED_LOCAL_PROOF_FIELDS,
            required_local_proof_output_ids: &CONTAMINATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
            required_local_proof_command_fragments:
                &CONTAMINATION_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS,
            report_section_id: "ancient_signal",
            summary_table_id: "damage_authenticity",
            report_anchor_tool_id: "schmutzi",
        },
        BamBindingSpec {
            stage_id: BAM_SEX,
            tool_id: "rxy",
            required_output_ids: &SEX_REQUIRED_OUTPUT_IDS,
            normalized_metrics_output_id: "sex_report",
            schema_extension_id: "bam_sex_normalized_v1",
            expected_local_proof_kind: LOCAL_PROOF_KIND_SMOKE,
            expected_local_proof_tool_id: "rxy",
            required_local_proof_fields: &SEX_REQUIRED_LOCAL_PROOF_FIELDS,
            required_local_proof_output_ids: &SEX_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
            required_local_proof_command_fragments: &[],
            report_section_id: "sample_identity",
            summary_table_id: "identity_inference",
            report_anchor_tool_id: "rxy",
        },
        BamBindingSpec {
            stage_id: BAM_SEX,
            tool_id: "yleaf",
            required_output_ids: &SEX_REQUIRED_OUTPUT_IDS,
            normalized_metrics_output_id: "sex_report",
            schema_extension_id: "bam_sex_normalized_v1",
            expected_local_proof_kind: LOCAL_PROOF_KIND_SMOKE,
            expected_local_proof_tool_id: "rxy",
            required_local_proof_fields: &SEX_REQUIRED_LOCAL_PROOF_FIELDS,
            required_local_proof_output_ids: &SEX_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
            required_local_proof_command_fragments: &[],
            report_section_id: "sample_identity",
            summary_table_id: "identity_inference",
            report_anchor_tool_id: "rxy",
        },
    ];
    #[cfg(feature = "bam_downstream")]
    specs.push(BamBindingSpec {
        stage_id: BAM_HAPLOGROUPS,
        tool_id: "yleaf",
        required_output_ids: &HAPLOGROUPS_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "haplogroups",
        schema_extension_id: "bam_haplogroups_normalized_v1",
        expected_local_proof_kind: LOCAL_PROOF_KIND_READY_PLAN,
        expected_local_proof_tool_id: "yleaf",
        required_local_proof_fields: &HAPLOGROUPS_REQUIRED_LOCAL_PROOF_FIELDS,
        required_local_proof_output_ids: &HAPLOGROUPS_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
        required_local_proof_command_fragments: &HAPLOGROUPS_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS,
        report_section_id: "sample_identity",
        summary_table_id: "identity_inference",
        report_anchor_tool_id: "yleaf",
    });
    specs
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
