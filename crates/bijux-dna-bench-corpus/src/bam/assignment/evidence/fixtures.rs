use crate::bam::validate_bam_corpus_fixture_manifest_path;
use crate::bam::BamCorpusFixtureValidationReport;
use crate::bam::DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH;
use crate::bam::DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH;
use crate::bam::DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH;
use crate::damage::validate_bam_damage_fixture_manifest_path;
use crate::damage::BamDamageFixtureValidationReport;
use crate::damage::DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH;
use anyhow::Result;
use std::path::Path;
pub(in crate::bam::assignment) struct FixtureInputs {
    pub(in crate::bam::assignment) damage: BamDamageFixtureValidationReport,
    pub(in crate::bam::assignment) adna: BamCorpusFixtureValidationReport,
    pub(in crate::bam::assignment) genotyping: BamCorpusFixtureValidationReport,
    pub(in crate::bam::assignment) kinship: BamCorpusFixtureValidationReport,
}
pub(in crate::bam::assignment) fn load_fixture_inputs(repo_root: &Path) -> Result<FixtureInputs> {
    let damage_fixture = validate_bam_damage_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH),
    )?;
    let adna_bam_fixture = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH),
    )?;
    let genotyping_fixture = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH),
    )?;
    let kinship_fixture = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH),
    )?;

    Ok(FixtureInputs {
        damage: damage_fixture,
        adna: adna_bam_fixture,
        genotyping: genotyping_fixture,
        kinship: kinship_fixture,
    })
}
