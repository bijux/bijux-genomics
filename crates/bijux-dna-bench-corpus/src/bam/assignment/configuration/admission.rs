use crate::bam::assignment::admission::genotyping::ensure_stage_matches_genotyping_fixture;
use crate::bam::assignment::admission::kinship::ensure_stage_matches_kinship_fixture;
use crate::bam::assignment::admission::samples::ensure_stage_sample_matches_bam_fixture;
use crate::bam::assignment::admission::samples::ensure_stage_sample_matches_damage_fixture;
use crate::bam::assignment::configuration::io::load_governed_toml;
use crate::bam::assignment::configuration::models::LocalAuthenticityCase;
use crate::bam::assignment::configuration::models::LocalAuthenticityConfig;
use crate::bam::assignment::configuration::models::LocalContaminationConfig;
use crate::bam::assignment::configuration::models::LocalDamageCase;
use crate::bam::assignment::configuration::models::LocalDamageConfig;
use crate::bam::assignment::configuration::models::LocalGenotypingConfig;
use crate::bam::assignment::configuration::models::LocalHaplogroupsConfig;
use crate::bam::assignment::configuration::models::LocalKinshipConfig;
use crate::bam::assignment::configuration::models::LocalSexCase;
use crate::bam::assignment::configuration::models::LocalSexConfig;
use crate::bam::assignment::declarations::DEFAULT_LOCAL_AUTHENTICITY_CONFIG_PATH;
use crate::bam::assignment::declarations::DEFAULT_LOCAL_CONTAMINATION_CONFIG_PATH;
use crate::bam::assignment::declarations::DEFAULT_LOCAL_DAMAGE_CONFIG_PATH;
use crate::bam::assignment::declarations::DEFAULT_LOCAL_GENOTYPING_CONFIG_PATH;
use crate::bam::assignment::declarations::DEFAULT_LOCAL_HAPLOGROUPS_CONFIG_PATH;
use crate::bam::assignment::declarations::DEFAULT_LOCAL_KINSHIP_CONFIG_PATH;
use crate::bam::assignment::declarations::DEFAULT_LOCAL_SEX_CONFIG_PATH;
use crate::bam::assignment::evidence::fixtures::FixtureInputs;
use crate::bam::BamCorpusFixtureGenotypingContractReport;
use crate::bam::BamCorpusFixtureKinshipContractReport;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_core::id_catalog::BAM_AUTHENTICITY;
use bijux_dna_core::id_catalog::BAM_CONTAMINATION;
use bijux_dna_core::id_catalog::BAM_DAMAGE;
use bijux_dna_core::id_catalog::BAM_GENOTYPING;
use bijux_dna_core::id_catalog::BAM_HAPLOGROUPS;
use bijux_dna_core::id_catalog::BAM_KINSHIP;
use bijux_dna_core::id_catalog::BAM_SEX;
use std::path::Path;
pub(in crate::bam::assignment) struct AdmittedConfigurations {
    pub(in crate::bam::assignment) authenticity_case: LocalAuthenticityCase,
    pub(in crate::bam::assignment) damage_case: LocalDamageCase,
    pub(in crate::bam::assignment) contamination: LocalContaminationConfig,
    pub(in crate::bam::assignment) sex_case: LocalSexCase,
    pub(in crate::bam::assignment) haplogroups: LocalHaplogroupsConfig,
    pub(in crate::bam::assignment) genotyping: LocalGenotypingConfig,
    pub(in crate::bam::assignment) genotyping_contract: BamCorpusFixtureGenotypingContractReport,
    pub(in crate::bam::assignment) kinship: LocalKinshipConfig,
    pub(in crate::bam::assignment) kinship_contract: BamCorpusFixtureKinshipContractReport,
}
pub(in crate::bam::assignment) fn load_admitted_configurations(
    repo_root: &Path,
    fixtures: &FixtureInputs,
) -> Result<AdmittedConfigurations> {
    let FixtureInputs {
        damage: damage_fixture,
        adna: adna_bam_fixture,
        genotyping: genotyping_fixture,
        kinship: kinship_fixture,
    } = fixtures;
    let authenticity = load_governed_toml::<LocalAuthenticityConfig>(
        &repo_root.join(DEFAULT_LOCAL_AUTHENTICITY_CONFIG_PATH),
        "BAM authenticity config",
    )?;
    let authenticity_case = authenticity
        .cases
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("BAM authenticity config must declare at least one case"))?;
    ensure_stage_sample_matches_damage_fixture(
        BAM_AUTHENTICITY,
        &authenticity_case.sample_id,
        damage_fixture,
    )?;

    let damage = load_governed_toml::<LocalDamageConfig>(
        &repo_root.join(DEFAULT_LOCAL_DAMAGE_CONFIG_PATH),
        "BAM damage config",
    )?;
    let damage_case = damage
        .cases
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("BAM damage config must declare at least one case"))?;
    ensure_stage_sample_matches_damage_fixture(BAM_DAMAGE, &damage_case.sample_id, damage_fixture)?;

    let contamination = load_governed_toml::<LocalContaminationConfig>(
        &repo_root.join(DEFAULT_LOCAL_CONTAMINATION_CONFIG_PATH),
        "BAM contamination config",
    )?;
    ensure_stage_sample_matches_bam_fixture(
        BAM_CONTAMINATION,
        &contamination.sample_id,
        adna_bam_fixture,
    )?;

    let sex = load_governed_toml::<LocalSexConfig>(
        &repo_root.join(DEFAULT_LOCAL_SEX_CONFIG_PATH),
        "BAM sex config",
    )?;
    let sex_case = sex
        .cases
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("BAM sex config must declare at least one case"))?;
    ensure_stage_sample_matches_bam_fixture(BAM_SEX, &sex_case.sample_id, adna_bam_fixture)?;

    let haplogroups = load_governed_toml::<LocalHaplogroupsConfig>(
        &repo_root.join(DEFAULT_LOCAL_HAPLOGROUPS_CONFIG_PATH),
        "BAM haplogroups config",
    )?;
    ensure_stage_sample_matches_bam_fixture(
        BAM_HAPLOGROUPS,
        &haplogroups.sample_id,
        adna_bam_fixture,
    )?;
    let genotyping = load_governed_toml::<LocalGenotypingConfig>(
        &repo_root.join(DEFAULT_LOCAL_GENOTYPING_CONFIG_PATH),
        "BAM genotyping config",
    )?;
    let genotyping_contract = ensure_stage_matches_genotyping_fixture(
        repo_root,
        BAM_GENOTYPING,
        &genotyping,
        genotyping_fixture,
    )?;
    let kinship = load_governed_toml::<LocalKinshipConfig>(
        &repo_root.join(DEFAULT_LOCAL_KINSHIP_CONFIG_PATH),
        "BAM kinship config",
    )?;
    let kinship_contract =
        ensure_stage_matches_kinship_fixture(repo_root, BAM_KINSHIP, &kinship, kinship_fixture)?;

    Ok(AdmittedConfigurations {
        authenticity_case,
        damage_case,
        contamination,
        sex_case,
        haplogroups,
        genotyping,
        genotyping_contract,
        kinship,
        kinship_contract,
    })
}
