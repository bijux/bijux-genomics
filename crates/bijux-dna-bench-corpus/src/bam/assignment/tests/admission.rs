#![cfg(test)]
use super::super::{
    admission::{
        genotyping::ensure_stage_matches_genotyping_fixture,
        kinship::ensure_stage_matches_kinship_fixture,
    },
    configuration::admission::load_admitted_configurations,
    evidence::fixtures::load_fixture_inputs,
};
use super::repo_root;
use anyhow::{Context, Result};
use bijux_dna_core::id_catalog::{BAM_GENOTYPING, BAM_KINSHIP};

#[test]
fn governed_fixtures_admit_seven_stage_configurations_in_the_corpus_owner() -> Result<()> {
    let root = repo_root()?;
    let fixtures = load_fixture_inputs(&root)?;
    let configs = load_admitted_configurations(&root, &fixtures)?;
    assert_eq!(configs.genotyping.sample_id, configs.genotyping_contract.sample_id);
    assert_eq!(configs.kinship.cases.len(), configs.kinship_contract.cases.len());
    assert_eq!(configs.authenticity_case.sample_id, fixtures.damage.sample_id);
    assert_eq!(configs.damage_case.sample_id, fixtures.damage.sample_id);
    Ok(())
}

#[test]
fn genotyping_rejects_a_nonfinite_posterior_against_governed_truth() -> Result<()> {
    let root = repo_root()?;
    let fixtures = load_fixture_inputs(&root)?;
    let mut configs = load_admitted_configurations(&root, &fixtures)?;
    configs.genotyping.min_posterior = f64::NAN;
    let error = ensure_stage_matches_genotyping_fixture(
        &root,
        BAM_GENOTYPING,
        &configs.genotyping,
        &fixtures.genotyping,
    )
    .err()
    .context("nonfinite posterior must reject")?;
    assert!(error.to_string().contains("field `min_posterior`"));
    Ok(())
}

#[test]
fn genotyping_rejects_an_unknown_sample_before_its_artifact_paths() -> Result<()> {
    let root = repo_root()?;
    let fixtures = load_fixture_inputs(&root)?;
    let mut configs = load_admitted_configurations(&root, &fixtures)?;
    configs.genotyping.sample_id = "foreign-sample".to_string();
    configs.genotyping.reference_fasta = root.join("absent-reference.fasta");
    let error = ensure_stage_matches_genotyping_fixture(
        &root,
        BAM_GENOTYPING,
        &configs.genotyping,
        &fixtures.genotyping,
    )
    .err()
    .context("unknown sample must reject")?;
    assert!(error.to_string().contains("sample `foreign-sample` is missing from BAM fixture"));
    Ok(())
}

#[test]
fn kinship_rejects_reference_build_substitution() -> Result<()> {
    let root = repo_root()?;
    let fixtures = load_fixture_inputs(&root)?;
    let mut configs = load_admitted_configurations(&root, &fixtures)?;
    configs.kinship.cases.first_mut().context("governed kinship case")?.reference_build =
        "incompatible-build".to_string();
    let error = ensure_stage_matches_kinship_fixture(
        &root,
        BAM_KINSHIP,
        &configs.kinship,
        &fixtures.kinship,
    )
    .err()
    .context("incompatible reference must reject")?;
    assert!(error.to_string().contains("reference_build `incompatible-build` must match"));
    Ok(())
}

#[test]
fn kinship_rejects_pairwise_relationship_loss() -> Result<()> {
    let root = repo_root()?;
    let fixtures = load_fixture_inputs(&root)?;
    let mut configs = load_admitted_configurations(&root, &fixtures)?;
    configs
        .kinship
        .cases
        .iter_mut()
        .find(|case| !case.expected_pairwise_results.is_empty())
        .context("governed related pair")?
        .expected_pairwise_results
        .clear();
    let error = ensure_stage_matches_kinship_fixture(
        &root,
        BAM_KINSHIP,
        &configs.kinship,
        &fixtures.kinship,
    )
    .err()
    .context("relationship loss must reject")?;
    assert!(error.to_string().contains("relationship labels"));
    Ok(())
}
