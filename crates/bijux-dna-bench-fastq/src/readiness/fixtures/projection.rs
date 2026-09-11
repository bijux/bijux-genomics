use super::model::{FastqParserFixtureCoverageRow, FastqParserFixtureCoverageStatus};
use super::parsing::validate_fixture_case;
use super::FIXTURE_REFERENCE_KIND;
use anyhow::Result;
use bijux_dna_domain_fastq::{find_fastq_parser_fixture_binding, find_fastq_parser_fixture_case};
use std::collections::BTreeSet;

/// Evaluate admitted FASTQ readiness inputs.
///
/// # Errors
/// Rejects invalid identities, incompatible evidence or failed publication.
pub fn collect_fastq_parser_fixture_coverage_rows(
    active_rows: Vec<super::super::active::FastqActiveStageToolMatrixRow>,
) -> Result<(usize, usize, Vec<FastqParserFixtureCoverageRow>)> {
    super::super::active::ensure_fastq_active_stage_tool_matrix_contract(&active_rows)?;
    let stage_count =
        active_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count =
        active_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in active_rows {
        let binding = find_fastq_parser_fixture_binding(&active_row.stage_id, &active_row.tool_id);
        let mut parser_fixture_parser_id = String::new();
        let mut parser_fixture_schema_id = String::new();
        let mut parser_fixture_reference = String::new();
        let mut parser_fixture_surface = String::new();
        let mut parser_fixture_canonical_tool_id = String::new();

        let (coverage_status, reason) = match binding {
            Some(binding) => {
                parser_fixture_parser_id = binding.parser_id.to_string();
                parser_fixture_schema_id = binding.parser_schema_id.to_string();
                parser_fixture_reference = binding.fixture_case_id.to_string();
                match find_fastq_parser_fixture_case(binding.fixture_case_id) {
                    Some(case) => {
                        parser_fixture_surface = case.semantic_surface.to_string();
                        parser_fixture_canonical_tool_id = case.canonical_tool_id.to_string();
                        match validate_fixture_case(binding.parser_id, binding.parser_schema_id, case) {
                            Ok(()) => (
                                FastqParserFixtureCoverageStatus::Covered,
                                format!(
                                    "active row `{}` / `{}` is governed by FASTQ parser fixture case `{}` using parser `{}` over canonical `{}` {} data",
                                    active_row.stage_id,
                                    active_row.tool_id,
                                    case.fixture_case_id,
                                    binding.parser_id,
                                    case.canonical_tool_id,
                                    case.semantic_surface
                                ),
                            ),
                            Err(error) => (
                                FastqParserFixtureCoverageStatus::InvalidFixtureCase,
                                format!(
                                    "active row `{}` / `{}` maps to invalid FASTQ parser fixture case `{}`: {}",
                                    active_row.stage_id, active_row.tool_id, case.fixture_case_id, error
                                ),
                            ),
                        }
                    }
                    None => (
                        FastqParserFixtureCoverageStatus::MissingFixtureCase,
                        format!(
                            "active row `{}` / `{}` is missing governed FASTQ parser fixture case `{}`",
                            active_row.stage_id, active_row.tool_id, binding.fixture_case_id
                        ),
                    ),
                }
            }
            None => (
                FastqParserFixtureCoverageStatus::MissingFixtureBinding,
                format!(
                    "active row `{}` / `{}` is missing a governed FASTQ parser fixture binding",
                    active_row.stage_id, active_row.tool_id
                ),
            ),
        };

        rows.push(FastqParserFixtureCoverageRow {
            stage_id: active_row.stage_id,
            tool_id: active_row.tool_id,
            corpus_id: active_row.corpus_id,
            asset_profile_id: active_row.asset_profile_id,
            adapter_id: active_row.adapter_id,
            parser_id: active_row.parser_id,
            schema_id: active_row.schema_id,
            parser_fixture_parser_id,
            parser_fixture_schema_id,
            parser_fixture_reference_kind: FIXTURE_REFERENCE_KIND.to_string(),
            parser_fixture_reference,
            parser_fixture_surface,
            parser_fixture_canonical_tool_id,
            coverage_status,
            reason,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    Ok((stage_count, tool_count, rows))
}
