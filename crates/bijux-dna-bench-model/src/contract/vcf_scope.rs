//! Governed VCF scope identity and cross-surface binding coverage admission.
use crate::{
    AllDomainActiveStageToolMatrixReport, AllDomainExpectedResultCoverageReport,
    AllDomainLocalJobCoverageReport, AllDomainNoPlaceholderCommandCheckReport,
    AllDomainOutputContractCoverageReport, AllDomainParserFixtureCoverageReport,
    AllDomainReportMapCoverageReport, RemovedFromScopeReport,
};
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;

const COVERAGE_STATUS_COVERED: &str = "covered";
const VCF_IMPUTE_STAGE_ID: &str = "vcf.impute";
const VCF_IMPUTATION_METRICS_STAGE_ID: &str = "vcf.imputation_metrics";
const VCF_LEGACY_IMPUTATION_STAGE_ID: &str = "vcf.imputation";
const VCF_POSTPROCESS_STAGE_ID: &str = "vcf.postprocess";
const VCF_OUTPUT_PARSER_ID: &str = "vcf.parser.vcf_output";
const VCF_REPORT_JSON_PARSER_ID: &str = "vcf.parser.report_json";
const VCF_IMPUTE_SCHEMA_ID: &str = "bijux.schemas.bench.vcf-normalized-metrics.impute.v1";
const VCF_IMPUTATION_METRICS_SCHEMA_ID: &str =
    "bijux.schemas.bench.vcf-normalized-metrics.imputation-metrics.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

/// Admit distinct VCF imputation and imputation-metrics parser/schema identities.
///
/// # Errors
/// Rejects missing observations, legacy stage names or contradictory parser/schema bindings.
pub fn validate_vcf_scope_imputation_identity(
    active_stage_tool_matrix: Option<&AllDomainActiveStageToolMatrixReport>,
    removed_from_scope: Option<&RemovedFromScopeReport>,
) -> Result<String> {
    let active_stage_tool_matrix = active_stage_tool_matrix
        .context("vcf imputation identity check requires the active stage-tool matrix")?;
    let removed_from_scope =
        removed_from_scope.context("vcf imputation identity check requires removed-from-scope")?;

    let active_rows =
        active_stage_tool_matrix.rows.iter().filter(|row| row.domain == "vcf").collect::<Vec<_>>();
    let removed_rows =
        removed_from_scope.rows.iter().filter(|row| row.domain == "vcf").collect::<Vec<_>>();

    let legacy_imputation_row_count = active_rows
        .iter()
        .filter(|row| row.stage_id == VCF_LEGACY_IMPUTATION_STAGE_ID)
        .count()
        + removed_rows.iter().filter(|row| row.stage_id == VCF_LEGACY_IMPUTATION_STAGE_ID).count();
    if legacy_imputation_row_count != 0 {
        return Err(anyhow!(
            "legacy VCF stage id `{VCF_LEGACY_IMPUTATION_STAGE_ID}` is still present in governed active-scope surfaces"
        ));
    }

    let impute_row_count = count_stage_rows(
        active_rows.iter().copied(),
        removed_rows.iter().copied(),
        VCF_IMPUTE_STAGE_ID,
    );
    let imputation_metrics_row_count = count_stage_rows(
        active_rows.iter().copied(),
        removed_rows.iter().copied(),
        VCF_IMPUTATION_METRICS_STAGE_ID,
    );
    if impute_row_count == 0 {
        return Err(anyhow!(
            "normalized VCF stage id `{VCF_IMPUTE_STAGE_ID}` is missing from governed active-scope surfaces"
        ));
    }
    if imputation_metrics_row_count == 0 {
        return Err(anyhow!(
            "normalized VCF stage id `{VCF_IMPUTATION_METRICS_STAGE_ID}` is missing from governed active-scope surfaces"
        ));
    }

    for (stage, parser, schema) in active_rows
        .iter()
        .map(|row| (row.stage_id.as_str(), row.parser_id.as_str(), row.schema_id.as_str()))
        .chain(
            removed_rows
                .iter()
                .map(|row| (row.stage_id.as_str(), row.parser_id.as_str(), row.schema_id.as_str())),
        )
    {
        validate_imputation_row(stage, parser, schema)?;
    }

    Ok(format!(
        "legacy_imputation_row_count={legacy_imputation_row_count}, impute_row_count={impute_row_count}, imputation_metrics_row_count={imputation_metrics_row_count}"
    ))
}

/// Borrowed scope observations; missing producers remain explicit admission failures.
#[derive(Clone, Copy, Default)]
pub struct VcfScopeCoverageEvidence<'a> {
    pub active_stage_tool_matrix: Option<&'a AllDomainActiveStageToolMatrixReport>,
    pub removed_from_scope: Option<&'a RemovedFromScopeReport>,
    pub parser_fixture_coverage: Option<&'a AllDomainParserFixtureCoverageReport>,
    pub output_contract_coverage: Option<&'a AllDomainOutputContractCoverageReport>,
    pub expected_result_coverage: Option<&'a AllDomainExpectedResultCoverageReport>,
    pub report_map_coverage: Option<&'a AllDomainReportMapCoverageReport>,
    pub local_job_coverage: Option<&'a AllDomainLocalJobCoverageReport>,
    pub no_placeholder_command_check: Option<&'a AllDomainNoPlaceholderCommandCheckReport>,
}

/// Require every active VCF postprocess binding across all declared coverage surfaces.
///
/// # Errors
/// Rejects absent observations, removed active bindings or missing covered identities.
pub fn validate_vcf_scope_postprocess_closure(
    evidence: VcfScopeCoverageEvidence<'_>,
) -> Result<String> {
    let VcfScopeCoverageEvidence {
        active_stage_tool_matrix,
        removed_from_scope,
        parser_fixture_coverage,
        output_contract_coverage,
        expected_result_coverage,
        report_map_coverage,
        local_job_coverage,
        no_placeholder_command_check,
    } = evidence;
    let active_stage_tool_matrix = active_stage_tool_matrix
        .context("vcf postprocess closure check requires the active stage-tool matrix")?;
    let removed_from_scope =
        removed_from_scope.context("vcf postprocess closure check requires removed-from-scope")?;
    let parser_fixture_coverage = parser_fixture_coverage
        .context("vcf postprocess closure check requires parser fixture coverage")?;
    let output_contract_coverage = output_contract_coverage
        .context("vcf postprocess closure check requires output-contract coverage")?;
    let expected_result_coverage = expected_result_coverage
        .context("vcf postprocess closure check requires expected-result coverage")?;
    let report_map_coverage = report_map_coverage
        .context("vcf postprocess closure check requires report-map coverage")?;
    let local_job_coverage =
        local_job_coverage.context("vcf postprocess closure check requires local-job coverage")?;
    let no_placeholder_command_check = no_placeholder_command_check
        .context("vcf postprocess closure check requires no-placeholder command coverage")?;

    let active_postprocess_keys = active_stage_tool_matrix
        .rows
        .iter()
        .filter(|row| row.domain == "vcf" && row.stage_id == VCF_POSTPROCESS_STAGE_ID)
        .map(binding_key_from_active_row)
        .collect::<BTreeSet<_>>();
    if active_postprocess_keys.is_empty() {
        return Err(anyhow!("`{VCF_POSTPROCESS_STAGE_ID}` must remain inside final active scope"));
    }

    let removed_postprocess_keys = removed_from_scope
        .rows
        .iter()
        .filter(|row| row.domain == "vcf" && row.stage_id == VCF_POSTPROCESS_STAGE_ID)
        .map(binding_key_from_removed_row)
        .collect::<BTreeSet<_>>();
    if !removed_postprocess_keys.is_empty() {
        return Err(anyhow!(
            "`{VCF_POSTPROCESS_STAGE_ID}` cannot be both active and removed from scope"
        ));
    }

    let surfaces = postprocess_coverage_sets(
        parser_fixture_coverage,
        output_contract_coverage,
        expected_result_coverage,
        report_map_coverage,
        local_job_coverage,
        no_placeholder_command_check,
    );
    for (surface, covered) in &surfaces {
        ensure_covering_keys(VCF_POSTPROCESS_STAGE_ID, surface, &active_postprocess_keys, covered)?;
    }
    let [parser_keys, output_keys, expected_keys, report_keys, local_job_keys, command_check_keys] =
        surfaces.map(|(_, keys)| keys.len());

    Ok(format!(
        "active_postprocess_row_count={}, parser_fixture_match_count={}, output_contract_match_count={}, expected_result_match_count={}, report_map_match_count={}, local_job_match_count={}, command_audit_match_count={}",
        active_postprocess_keys.len(),
        parser_keys,
        output_keys,
        expected_keys,
        report_keys,
        local_job_keys,
        command_check_keys
    ))
}

fn postprocess_coverage_sets(
    parser_fixture_coverage: &AllDomainParserFixtureCoverageReport,
    output_contract_coverage: &AllDomainOutputContractCoverageReport,
    expected_result_coverage: &AllDomainExpectedResultCoverageReport,
    report_map_coverage: &AllDomainReportMapCoverageReport,
    local_job_coverage: &AllDomainLocalJobCoverageReport,
    no_placeholder_command_check: &AllDomainNoPlaceholderCommandCheckReport,
) -> [(&'static str, BTreeSet<BindingKey>); 6] {
    [
        (
            "parser fixture coverage",
            parser_fixture_coverage
                .rows
                .iter()
                .filter(|row| {
                    row.domain == "vcf"
                        && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                        && row.coverage_status == COVERAGE_STATUS_COVERED
                })
                .map(binding_key_from_parser_row)
                .collect::<BTreeSet<_>>(),
        ),
        (
            "output-contract coverage",
            output_contract_coverage
                .rows
                .iter()
                .filter(|row| {
                    row.domain == "vcf"
                        && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                        && row.coverage_status == COVERAGE_STATUS_COVERED
                })
                .map(binding_key_from_output_contract_row)
                .collect::<BTreeSet<_>>(),
        ),
        (
            "expected-result coverage",
            expected_result_coverage
                .rows
                .iter()
                .filter(|row| {
                    row.domain == "vcf"
                        && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                        && row.coverage_status == COVERAGE_STATUS_COVERED
                })
                .map(binding_key_from_expected_result_row)
                .collect::<BTreeSet<_>>(),
        ),
        (
            "report-map coverage",
            report_map_coverage
                .rows
                .iter()
                .filter(|row| {
                    row.domain == "vcf"
                        && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                        && row.coverage_status == COVERAGE_STATUS_COVERED
                })
                .map(binding_key_from_report_map_row)
                .collect::<BTreeSet<_>>(),
        ),
        (
            "local-job coverage",
            local_job_coverage
                .rows
                .iter()
                .filter(|row| {
                    row.domain == "vcf"
                        && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                        && row.coverage_status == COVERAGE_STATUS_COVERED
                })
                .map(binding_key_from_local_job_row)
                .collect::<BTreeSet<_>>(),
        ),
        (
            "no-placeholder command check",
            no_placeholder_command_check
                .rows
                .iter()
                .filter(|row| {
                    row.domain == "vcf" && row.stage_id == VCF_POSTPROCESS_STAGE_ID && row.ok
                })
                .map(binding_key_from_no_placeholder_row)
                .collect::<BTreeSet<_>>(),
        ),
    ]
}

fn ensure_covering_keys(
    stage_id: &str,
    surface_name: &str,
    active_keys: &BTreeSet<BindingKey>,
    covered_keys: &BTreeSet<BindingKey>,
) -> Result<()> {
    let missing_keys =
        active_keys.difference(covered_keys).map(render_binding_key).collect::<Vec<_>>();
    if !missing_keys.is_empty() {
        return Err(anyhow!(
            "`{}` is missing governed {} for {}",
            stage_id,
            surface_name,
            missing_keys.join(", ")
        ));
    }
    Ok(())
}

fn count_stage_rows<'a, I, J>(active_rows: I, removed_rows: J, stage_id: &str) -> usize
where
    I: Iterator<Item = &'a crate::AllDomainActiveStageToolMatrixRow>,
    J: Iterator<Item = &'a crate::RemovedFromScopeRow>,
{
    active_rows.filter(|row| row.stage_id == stage_id).count()
        + removed_rows.filter(|row| row.stage_id == stage_id).count()
}

fn binding_key_from_active_row(row: &crate::AllDomainActiveStageToolMatrixRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_removed_row(row: &crate::RemovedFromScopeRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_parser_row(row: &crate::AllDomainParserFixtureCoverageRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_output_contract_row(
    row: &crate::AllDomainOutputContractCoverageRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_expected_result_row(
    row: &crate::AllDomainExpectedResultCoverageRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_report_map_row(row: &crate::AllDomainReportMapCoverageRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_local_job_row(row: &crate::AllDomainLocalJobCoverageRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_no_placeholder_row(
    row: &crate::AllDomainNoPlaceholderCommandRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn render_binding_key(key: &BindingKey) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        key.domain, key.stage_id, key.tool_id, key.corpus_id, key.asset_profile_id
    )
}

fn validate_imputation_row(stage: &str, parser: &str, schema: &str) -> Result<()> {
    let (expected_parser, expected_schema) = match stage {
        VCF_IMPUTE_STAGE_ID => (VCF_OUTPUT_PARSER_ID, VCF_IMPUTE_SCHEMA_ID),
        VCF_IMPUTATION_METRICS_STAGE_ID => {
            (VCF_REPORT_JSON_PARSER_ID, VCF_IMPUTATION_METRICS_SCHEMA_ID)
        }
        _ => return Ok(()),
    };
    if parser != expected_parser || schema != expected_schema {
        return Err(anyhow!("`{stage}` must keep parser `{expected_parser}` and schema `{expected_schema}`; found parser `{parser}` and schema `{schema}`"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/contracts/vcf_scope.rs"]
mod tests;
