use crate::bam::assignment::admission::identity::ensure_paths_match;
use crate::bam::assignment::admission::samples::ensure_stage_sample_matches_bam_fixture;
use crate::bam::assignment::admission::samples::sample_alignment_path;
use crate::bam::assignment::configuration::models::LocalKinshipCase;
use crate::bam::assignment::configuration::models::LocalKinshipConfig;
use crate::bam::BamCorpusFixtureKinshipContractReport;
use crate::bam::BamCorpusFixtureValidationReport;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
pub(in crate::bam::assignment) fn ensure_stage_matches_kinship_fixture(
    repo_root: &Path,
    stage_id: &str,
    config: &LocalKinshipConfig,
    fixture: &BamCorpusFixtureValidationReport,
) -> Result<BamCorpusFixtureKinshipContractReport> {
    let contract = fixture.kinship_contract.clone().ok_or_else(|| {
        anyhow!("BAM stage `{stage_id}` requires a governed kinship contract in the fixture")
    })?;
    if config.cases.len() != contract.cases.len() {
        return Err(anyhow!(
            "BAM stage `{stage_id}` must declare {} kinship cases, found {}",
            contract.cases.len(),
            config.cases.len()
        ));
    }

    let contract_cases = contract
        .cases
        .iter()
        .map(|case| (case.sample_id.clone(), case))
        .collect::<BTreeMap<_, _>>();
    for case in &config.cases {
        ensure_stage_sample_matches_bam_fixture(stage_id, &case.sample_id, fixture)?;
        let expected = contract_cases.get(&case.sample_id).ok_or_else(|| {
            anyhow!(
                "BAM stage `{stage_id}` sample `{}` is missing from the kinship contract",
                case.sample_id
            )
        })?;
        admit_case(repo_root, stage_id, case, fixture, &contract, expected)?;
    }

    Ok(contract)
}

fn admit_case(
    repo_root: &Path,
    stage_id: &str,
    case: &LocalKinshipCase,
    fixture: &BamCorpusFixtureValidationReport,
    contract: &BamCorpusFixtureKinshipContractReport,
    expected: &crate::bam::BamCorpusFixtureKinshipCaseReport,
) -> Result<()> {
    ensure_paths_match(
        repo_root,
        stage_id,
        &format!("{}.bam", case.sample_id),
        &case.bam,
        &repo_root.join(sample_alignment_path(fixture, &case.sample_id)?),
    )?;
    if case.reference_panel != contract.reference_panel {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` reference_panel `{}` must match `{}`",
            case.sample_id,
            case.reference_panel,
            contract.reference_panel
        ));
    }
    if case.reference_build != contract.reference_build {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` reference_build `{}` must match `{}`",
            case.sample_id,
            case.reference_build,
            contract.reference_build
        ));
    }
    if case.population_scope != contract.population_scope {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` population_scope `{}` must match `{}`",
            case.sample_id,
            case.population_scope,
            contract.population_scope
        ));
    }
    if case.min_overlap_snps != expected.min_overlap_snps {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` min_overlap_snps `{}` must match `{}`",
            case.sample_id,
            case.min_overlap_snps,
            expected.min_overlap_snps
        ));
    }
    if case.expected_status != expected.expected_status {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` expected_status `{}` must match `{}`",
            case.sample_id,
            case.expected_status,
            expected.expected_status
        ));
    }
    if case.expected_observed_max_overlap_snps != expected.expected_observed_max_overlap_snps {
        return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` expected_observed_max_overlap_snps `{}` must match `{}`",
                case.sample_id,
                case.expected_observed_max_overlap_snps,
                expected.expected_observed_max_overlap_snps
            ));
    }
    let relationship_labels = case
        .expected_pairwise_results
        .iter()
        .map(|result| result.relationship_label.clone())
        .collect::<Vec<_>>();
    if relationship_labels != expected.expected_relationship_labels {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` relationship labels {:?} must match {:?}",
            case.sample_id,
            relationship_labels,
            expected.expected_relationship_labels
        ));
    }
    let inferred_skip_behavior = if relationship_labels.is_empty() {
        "stop_without_pairwise_results"
    } else {
        "emit_pairwise_results"
    };
    if inferred_skip_behavior != expected.skip_behavior {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` skip behavior `{}` must match `{}`",
            case.sample_id,
            inferred_skip_behavior,
            expected.skip_behavior
        ));
    }

    Ok(())
}
