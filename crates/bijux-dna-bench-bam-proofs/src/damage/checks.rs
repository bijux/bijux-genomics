use super::contracts::float_matches;
use super::declarations::EXPECTED_DAMAGE_SIGNAL;
use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_METHOD;
use super::declarations::EXPECTED_PARSER_OUTPUT_SCHEMA_VERSION;
use super::declarations::EXPECTED_SAMPLE_ID;
use super::declarations::EXPECTED_SHORT_FRAGMENT_FRACTION;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::EXPECTED_STRICT_PROFILE_UPGRADED;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::declarations::EXPECTED_TERMINAL_C_TO_T_5P;
use super::declarations::EXPECTED_TERMINAL_G_TO_A_3P;
use super::observations::DamageStageProof;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyRow;
use std::collections::BTreeSet;
use std::path::Path;
pub(super) fn check_summary_ready(proof: &DamageStageProof) -> bool {
    proof.summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && proof.summary.stage_id == EXPECTED_STAGE_ID
        && float_matches(proof.summary.terminal_c_to_t_5p, EXPECTED_TERMINAL_C_TO_T_5P)
        && float_matches(proof.summary.terminal_g_to_a_3p, EXPECTED_TERMINAL_G_TO_A_3P)
        && float_matches(proof.summary.short_fragment_fraction, EXPECTED_SHORT_FRAGMENT_FRACTION)
        && proof.summary.damage_signal == EXPECTED_DAMAGE_SIGNAL
        && proof.summary.strict_profile_upgraded == EXPECTED_STRICT_PROFILE_UPGRADED
        && proof.summary.advisory_boundary.stage_id == EXPECTED_STAGE_ID
        && proof.summary.advisory_boundary.advisory_only
}

pub(super) fn check_unified_metrics_ready(
    proof: &DamageStageProof,
    local_tool_set: &BTreeSet<String>,
) -> bool {
    float_matches(proof.unified_metrics.canonical.c_to_t_5p, EXPECTED_TERMINAL_C_TO_T_5P)
        && float_matches(proof.unified_metrics.canonical.g_to_a_3p, EXPECTED_TERMINAL_G_TO_A_3P)
        && proof.unified_metrics.tools_seen.iter().cloned().collect::<BTreeSet<_>>()
            == *local_tool_set
        && proof.unified_metrics.comparison.is_some()
}

pub(super) fn check_parser_output_contract_ready(
    proof: &DamageStageProof,
    parsed_tool_id_set: &BTreeSet<String>,
    local_tool_set: &BTreeSet<String>,
) -> bool {
    proof.parser_output.schema_version == EXPECTED_PARSER_OUTPUT_SCHEMA_VERSION
        && proof.parser_output.stage_id == EXPECTED_STAGE_ID
        && !proof.parser_output.parsed_tools.is_empty()
        && parsed_tool_id_set == local_tool_set
        && proof.parser_output.parsed_tools.iter().all(|tool| {
            !tool.tool_id.is_empty()
                && tool.metrics.c_to_t_5p.is_finite()
                && tool.metrics.g_to_a_3p.is_finite()
        })
}

pub(super) fn check_normalized_metrics_ready(
    proof: &DamageStageProof,
    local_tool_set: &BTreeSet<String>,
) -> bool {
    proof.normalized_metrics.schema_version == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && proof.normalized_metrics.stage_id == EXPECTED_STAGE_ID
        && proof.normalized_metrics.sample_id == EXPECTED_SAMPLE_ID
        && proof.normalized_metrics.tool_id == EXPECTED_METHOD
        && proof.normalized_metrics.tools_seen.iter().cloned().collect::<BTreeSet<_>>()
            == *local_tool_set
        && float_matches(
            proof.normalized_metrics.expected_terminal_c_to_t_5p,
            EXPECTED_TERMINAL_C_TO_T_5P,
        )
        && float_matches(proof.normalized_metrics.terminal_c_to_t_5p, EXPECTED_TERMINAL_C_TO_T_5P)
        && float_matches(proof.normalized_metrics.terminal_c_to_t_5p_delta, 0.0)
        && float_matches(
            proof.normalized_metrics.expected_terminal_g_to_a_3p,
            EXPECTED_TERMINAL_G_TO_A_3P,
        )
        && float_matches(proof.normalized_metrics.terminal_g_to_a_3p, EXPECTED_TERMINAL_G_TO_A_3P)
        && float_matches(proof.normalized_metrics.terminal_g_to_a_3p_delta, 0.0)
        && float_matches(
            proof.normalized_metrics.expected_short_fragment_fraction,
            EXPECTED_SHORT_FRAGMENT_FRACTION,
        )
        && float_matches(
            proof.normalized_metrics.short_fragment_fraction,
            EXPECTED_SHORT_FRAGMENT_FRACTION,
        )
        && float_matches(proof.normalized_metrics.short_fragment_fraction_delta, 0.0)
        && proof.normalized_metrics.expected_damage_signal == EXPECTED_DAMAGE_SIGNAL
        && proof.normalized_metrics.damage_signal == EXPECTED_DAMAGE_SIGNAL
        && proof.normalized_metrics.expected_strict_profile_upgraded
            == EXPECTED_STRICT_PROFILE_UPGRADED
        && proof.normalized_metrics.strict_profile_upgraded == EXPECTED_STRICT_PROFILE_UPGRADED
        && proof.normalized_metrics.expectation_matched
}

pub(super) fn check_damage_metric_consistency_ready(
    repo_root: &Path,
    damage_row: &BamDamageAuthenticityReadyRow,
    proof: &DamageStageProof,
    local_tool_set: &BTreeSet<String>,
    inherited_artifact_set: &BTreeSet<String>,
    required_artifact_paths: &[String],
) -> bool {
    proof.local_smoke_report_path == repo_root.join(&damage_row.local_smoke_proof_path)
        && proof.local_smoke_report.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && proof.local_smoke_report.stage_id == EXPECTED_STAGE_ID
        && proof.local_smoke_report.sample_id == EXPECTED_SAMPLE_ID
        && proof.local_smoke_report.method == EXPECTED_METHOD
        && proof.local_smoke_report.expectation_matched
        && proof.local_smoke_report.tools_seen.iter().cloned().collect::<BTreeSet<_>>()
            == *local_tool_set
        && float_matches(proof.local_smoke_report.terminal_c_to_t_5p, EXPECTED_TERMINAL_C_TO_T_5P)
        && float_matches(proof.local_smoke_report.terminal_g_to_a_3p, EXPECTED_TERMINAL_G_TO_A_3P)
        && float_matches(
            proof.local_smoke_report.short_fragment_fraction,
            EXPECTED_SHORT_FRAGMENT_FRACTION,
        )
        && proof.local_smoke_report.damage_signal == EXPECTED_DAMAGE_SIGNAL
        && proof.local_smoke_report.strict_profile_upgraded == EXPECTED_STRICT_PROFILE_UPGRADED
        && required_artifact_paths.iter().all(|path| inherited_artifact_set.contains(path))
        && proof.local_smoke_report.damage_report.ends_with("damage.summary.json")
        && proof
            .local_smoke_report
            .terminal_position_metrics
            .ends_with("damage.unified_metrics.json")
        && proof.local_smoke_report.parser_output.ends_with("damage.parser_output.json")
        && proof.local_smoke_report.stage_metrics.ends_with("stage.metrics.json")
        && float_matches(
            proof.summary.terminal_c_to_t_5p,
            proof.local_smoke_report.terminal_c_to_t_5p,
        )
        && float_matches(
            proof.summary.terminal_g_to_a_3p,
            proof.local_smoke_report.terminal_g_to_a_3p,
        )
        && float_matches(
            proof.summary.short_fragment_fraction,
            proof.local_smoke_report.short_fragment_fraction,
        )
        && proof.summary.damage_signal == proof.local_smoke_report.damage_signal
        && proof.summary.strict_profile_upgraded == proof.local_smoke_report.strict_profile_upgraded
        && float_matches(
            proof.unified_metrics.canonical.c_to_t_5p,
            proof.local_smoke_report.terminal_c_to_t_5p,
        )
        && float_matches(
            proof.unified_metrics.canonical.g_to_a_3p,
            proof.local_smoke_report.terminal_g_to_a_3p,
        )
        && float_matches(
            proof.normalized_metrics.terminal_c_to_t_5p,
            proof.local_smoke_report.terminal_c_to_t_5p,
        )
        && float_matches(
            proof.normalized_metrics.terminal_g_to_a_3p,
            proof.local_smoke_report.terminal_g_to_a_3p,
        )
        && float_matches(
            proof.normalized_metrics.short_fragment_fraction,
            proof.local_smoke_report.short_fragment_fraction,
        )
        && proof.normalized_metrics.damage_signal == proof.local_smoke_report.damage_signal
        && proof.normalized_metrics.strict_profile_upgraded
            == proof.local_smoke_report.strict_profile_upgraded
}
