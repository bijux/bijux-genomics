use crate::bam::assignment::configuration::admission::load_admitted_configurations;
use crate::bam::assignment::evidence::fixtures::load_fixture_inputs;
use crate::bam::assignment::evidence::models::BamCorpusAssignmentContext;
use crate::bam::assignment::evidence::profiles::build_profiles;
use crate::bam::assignment::evidence::projection::authenticity::project_authenticity;
use crate::bam::assignment::evidence::projection::contamination::project_contamination;
use crate::bam::assignment::evidence::projection::damage::project_damage;
use crate::bam::assignment::evidence::projection::genotyping::project_genotyping;
use crate::bam::assignment::evidence::projection::haplogroups::project_haplogroups;
use crate::bam::assignment::evidence::projection::kinship::project_kinship;
use crate::bam::assignment::evidence::projection::sex::project_sex;
use anyhow::Result;
use bijux_dna_core::id_catalog::BAM_AUTHENTICITY;
use bijux_dna_core::id_catalog::BAM_CONTAMINATION;
use bijux_dna_core::id_catalog::BAM_DAMAGE;
use bijux_dna_core::id_catalog::BAM_GENOTYPING;
use bijux_dna_core::id_catalog::BAM_HAPLOGROUPS;
use bijux_dna_core::id_catalog::BAM_KINSHIP;
use bijux_dna_core::id_catalog::BAM_SEX;
use std::collections::BTreeMap;
use std::path::Path;
pub(in crate::bam::assignment) fn load_bam_corpus_assignment_context(
    repo_root: &Path,
) -> Result<BamCorpusAssignmentContext> {
    let fixtures = load_fixture_inputs(repo_root)?;
    let configs = load_admitted_configurations(repo_root, &fixtures)?;
    let profiles = build_profiles(&fixtures.damage, &fixtures.adna)?;
    let mut stage_evidence = BTreeMap::new();
    stage_evidence.insert(
        BAM_AUTHENTICITY.to_string(),
        project_authenticity(
            &configs.authenticity_case,
            &profiles.damage_expectation,
            &profiles.damage_assets,
        ),
    );
    stage_evidence.insert(
        BAM_DAMAGE.to_string(),
        project_damage(&configs.damage_case, &profiles.damage_expectation, &profiles.damage_assets),
    );
    stage_evidence.insert(
        BAM_CONTAMINATION.to_string(),
        project_contamination(&configs.contamination, &profiles.inherited_adna_profile)?,
    );
    stage_evidence.insert(
        BAM_SEX.to_string(),
        project_sex(&configs.sex_case, &profiles.inherited_adna_profile)?,
    );
    stage_evidence.insert(
        BAM_HAPLOGROUPS.to_string(),
        project_haplogroups(&configs.haplogroups, &profiles.inherited_adna_profile)?,
    );
    stage_evidence.insert(
        BAM_GENOTYPING.to_string(),
        project_genotyping(&configs.genotyping, &configs.genotyping_contract)?,
    );
    stage_evidence.insert(
        BAM_KINSHIP.to_string(),
        project_kinship(&fixtures.kinship, &configs.kinship, &configs.kinship_contract)?,
    );
    Ok(BamCorpusAssignmentContext { stage_evidence })
}
