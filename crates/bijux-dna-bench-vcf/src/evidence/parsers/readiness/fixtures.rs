use anyhow::{bail, Result};
use bijux_dna_domain_vcf::{vcf_parser_fixture_inventory, VcfDomainStage};
use std::{collections::BTreeSet, path::Path};
pub(super) fn validate_parser_fixture_family(
    repo_root: &Path,
    tool_id: &str,
    expected_stages: &[VcfDomainStage],
    detail: &str,
) -> Result<String> {
    let expected_stage_set =
        expected_stages.iter().map(|stage| stage.as_str().to_string()).collect::<BTreeSet<_>>();
    let rows = vcf_parser_fixture_inventory()
        .iter()
        .filter(|row| row.tool_id == tool_id && expected_stage_set.contains(row.stage.as_str()))
        .copied()
        .collect::<Vec<_>>();
    let actual_stage_set =
        rows.iter().map(|row| row.stage.as_str().to_string()).collect::<BTreeSet<_>>();
    let missing_stages =
        expected_stage_set.difference(&actual_stage_set).map(String::as_str).collect::<Vec<_>>();
    if !missing_stages.is_empty() {
        bail!(
            "VCF parser fixture family `{tool_id}` is missing stages [{}]",
            missing_stages.join(", ")
        );
    }

    for row in &rows {
        if row.parser_id.trim().is_empty() {
            bail!(
                "VCF parser fixture family `{tool_id}` is missing parser_id for stage `{}`",
                row.stage.as_str()
            );
        }
        let fixture_path = repo_root.join(row.fixture_path);
        if !fixture_path.exists() {
            bail!(
                "VCF parser fixture family `{tool_id}` is missing fixture path `{}`",
                row.fixture_path
            );
        }
    }
    if rows.len() != expected_stages.len() {
        bail!(
            "VCF parser fixture family `{tool_id}` expected {} governed rows for this goal slice but found {}",
            expected_stages.len(),
            rows.len()
        );
    }

    Ok(detail.to_string())
}
