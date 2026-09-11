use bijux_dna_core::id_catalog::{BAM_AUTHENTICITY, BAM_DAMAGE};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
pub(in crate::readiness) const BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_damage_authenticity_ready.v1";
pub(in crate::readiness) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(in crate::readiness) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(in crate::readiness) const CHECKED_SURFACE_COUNT: usize = 8;
pub(in crate::readiness) const EXPECTED_FIXTURE_ID: &str = "corpus-01-adna-damage-mini";
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
    pub(in crate::readiness) required_local_smoke_fields: &'static [&'static str],
    pub(in crate::readiness) report_section_id: &'static str,
    pub(in crate::readiness) summary_table_id: &'static str,
    pub(in crate::readiness) report_anchor_tool_id: &'static str,
}
pub(in crate::readiness) const DAMAGE_REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["damage_report", "terminal_position_metrics", "parser_output", "stage_metrics"];
pub(in crate::readiness) const AUTHENTICITY_REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["authenticity_report", "summary", "stage_metrics"];
pub(in crate::readiness) const DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 8] = [
    "terminal_c_to_t_5p",
    "terminal_g_to_a_3p",
    "short_fragment_fraction",
    "damage_signal",
    "damage_report",
    "terminal_position_metrics",
    "parser_output",
    "stage_metrics",
];
pub(in crate::readiness) const AUTHENTICITY_REQUIRED_LOCAL_SMOKE_FIELDS: [&str; 13] = [
    "score",
    "confidence",
    "pmd_like_signal_present",
    "consumed_metrics",
    "missing_metrics",
    "authenticity_report",
    "authenticity_summary",
    "authenticity_composite",
    "stage_metrics",
    "damage_unified_metrics",
    "contamination_summary",
    "coverage_regime",
    "mapping_summary",
];
pub(in crate::readiness) const BAM_BINDING_SPECS: [BamBindingSpec; 8] = [
    BamBindingSpec {
        stage_id: BAM_AUTHENTICITY,
        tool_id: "authenticct",
        required_output_ids: &AUTHENTICITY_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "authenticity_report",
        schema_extension_id: "bam_authenticity_normalized_v1",
        required_local_smoke_fields: &AUTHENTICITY_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "authenticct",
    },
    BamBindingSpec {
        stage_id: BAM_AUTHENTICITY,
        tool_id: "damageprofiler",
        required_output_ids: &AUTHENTICITY_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "authenticity_report",
        schema_extension_id: "bam_authenticity_normalized_v1",
        required_local_smoke_fields: &AUTHENTICITY_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "authenticct",
    },
    BamBindingSpec {
        stage_id: BAM_AUTHENTICITY,
        tool_id: "pmdtools",
        required_output_ids: &AUTHENTICITY_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "authenticity_report",
        schema_extension_id: "bam_authenticity_normalized_v1",
        required_local_smoke_fields: &AUTHENTICITY_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "authenticct",
    },
    BamBindingSpec {
        stage_id: BAM_DAMAGE,
        tool_id: "addeam",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: BAM_DAMAGE,
        tool_id: "damageprofiler",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: BAM_DAMAGE,
        tool_id: "mapdamage2",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: BAM_DAMAGE,
        tool_id: "pmdtools",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
    BamBindingSpec {
        stage_id: BAM_DAMAGE,
        tool_id: "pydamage",
        required_output_ids: &DAMAGE_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "damage_report",
        schema_extension_id: "bam_damage_normalized_v1",
        required_local_smoke_fields: &DAMAGE_REQUIRED_LOCAL_SMOKE_FIELDS,
        report_section_id: "ancient_signal",
        summary_table_id: "damage_authenticity",
        report_anchor_tool_id: "mapdamage2",
    },
];

pub(in crate::readiness) fn expected_stage_ids() -> BTreeSet<&'static str> {
    BAM_BINDING_SPECS.iter().map(|spec| spec.stage_id).collect()
}
pub(in crate::readiness) fn expected_tool_ids_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, Vec<String>>::new();
    for spec in BAM_BINDING_SPECS {
        by_stage.entry(spec.stage_id.to_string()).or_default().push(spec.tool_id.to_string());
    }
    by_stage
}
pub(in crate::readiness) fn required_output_ids_by_stage() -> BTreeMap<String, Vec<String>> {
    let mut by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for spec in BAM_BINDING_SPECS {
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
pub(in crate::readiness) fn required_local_smoke_fields_by_stage() -> BTreeMap<String, Vec<String>>
{
    let mut by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for spec in BAM_BINDING_SPECS {
        let stage_fields = by_stage.entry(spec.stage_id.to_string()).or_default();
        for field in spec.required_local_smoke_fields {
            stage_fields.insert((*field).to_string());
        }
    }
    by_stage
        .into_iter()
        .map(|(stage_id, fields)| (stage_id, fields.into_iter().collect()))
        .collect()
}
