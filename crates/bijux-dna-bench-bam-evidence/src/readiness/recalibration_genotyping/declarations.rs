#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::BAM_GENOTYPING;
use bijux_dna_core::id_catalog::BAM_RECALIBRATION;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
pub(in crate::readiness) const BAM_RECALIBRATION_GENOTYPING_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_recalibration_genotyping_ready.v1";
pub(in crate::readiness) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(in crate::readiness) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(in crate::readiness) const CHECKED_SURFACE_COUNT: usize = 8;
pub(in crate::readiness) const LOCAL_PROOF_KIND_SMOKE: &str = "local_smoke";
#[cfg(feature = "bam_downstream")]
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
    pub(in crate::readiness) fixture_id: &'static str,
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
pub(in crate::readiness) const RECALIBRATION_REQUIRED_OUTPUT_IDS: [&str; 5] =
    ["recal_bam", "recal_bai", "recal_report", "summary", "stage_metrics"];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const GENOTYPING_REQUIRED_OUTPUT_IDS: [&str; 3] =
    ["genotyping_report", "summary", "stage_metrics"];
pub(in crate::readiness) const RECALIBRATION_REQUIRED_LOCAL_PROOF_FIELDS: [&str; 14] = [
    "requested_mode",
    "effective_mode",
    "status",
    "reason",
    "known_sites",
    "coverage_gate",
    "observed_mean_coverage",
    "observed_breadth_1x",
    "output_bam_present",
    "recalibration_report_present",
    "recalibrated_bam",
    "recalibration_report",
    "recalibration_summary",
    "stage_metrics",
];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const GENOTYPING_REQUIRED_LOCAL_PROOF_FIELDS: [&str; 9] = [
    "sample_id",
    "producer_contract",
    "min_call_rate",
    "min_posterior",
    "reference",
    "regions",
    "sites",
    "caller",
    "tool",
];
pub(in crate::readiness) const RECALIBRATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS: [&str; 4] =
    ["recalibrated_bam", "recalibration_report", "recalibration_summary", "stage_metrics"];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const GENOTYPING_REQUIRED_LOCAL_PROOF_OUTPUT_IDS: [&str; 7] = [
    "genotyping_report",
    "summary",
    "stage_metrics",
    "genotyping_bcf",
    "genotyping_vcf",
    "genotyping_vcf_tbi",
    "genotyping_gl",
];
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) const GENOTYPING_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS: [&str; 7] = [
    "human_like_genotyping_candidate_panel.sam.bai",
    "corpus_01_bam_reference.fasta",
    "human_like_genotyping_candidate_sites.vcf",
    "human_like_genotyping_target_regions.txt",
    "artifacts/benchmark-smoke/readiness/local-ready/bam.genotyping/genotyping.bcf",
    "artifacts/benchmark-smoke/readiness/local-ready/bam.genotyping/genotyping.vcf.gz",
    "artifacts/benchmark-smoke/readiness/local-ready/bam.genotyping/genotyping.gl.json",
];
pub(in crate::readiness) fn bam_binding_specs() -> Vec<BamBindingSpec> {
    #[allow(unused_mut)]
    let mut specs = vec![BamBindingSpec {
        stage_id: BAM_RECALIBRATION,
        tool_id: "gatk",
        fixture_id: "corpus-01-bam-mini",
        required_output_ids: &RECALIBRATION_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "recal_report",
        schema_extension_id: "bam_recalibration_normalized_v1",
        expected_local_proof_kind: LOCAL_PROOF_KIND_SMOKE,
        expected_local_proof_tool_id: "gatk",
        required_local_proof_fields: &RECALIBRATION_REQUIRED_LOCAL_PROOF_FIELDS,
        required_local_proof_output_ids: &RECALIBRATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
        required_local_proof_command_fragments: &[],
        report_section_id: "downstream_readiness",
        summary_table_id: "variant_readiness",
        report_anchor_tool_id: "gatk",
    }];
    #[cfg(feature = "bam_downstream")]
    specs.push(BamBindingSpec {
        stage_id: BAM_GENOTYPING,
        tool_id: "bcftools",
        fixture_id: "corpus-01-genotyping-mini",
        required_output_ids: &GENOTYPING_REQUIRED_OUTPUT_IDS,
        normalized_metrics_output_id: "genotyping_report",
        schema_extension_id: "bam_genotyping_normalized_v1",
        expected_local_proof_kind: LOCAL_PROOF_KIND_READY_PLAN,
        expected_local_proof_tool_id: "bcftools",
        required_local_proof_fields: &GENOTYPING_REQUIRED_LOCAL_PROOF_FIELDS,
        required_local_proof_output_ids: &GENOTYPING_REQUIRED_LOCAL_PROOF_OUTPUT_IDS,
        required_local_proof_command_fragments: &GENOTYPING_REQUIRED_LOCAL_PROOF_COMMAND_FRAGMENTS,
        report_section_id: "downstream_readiness",
        summary_table_id: "variant_readiness",
        report_anchor_tool_id: "bcftools",
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
