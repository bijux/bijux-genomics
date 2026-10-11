use crate::bam::assignment::formatting::file_name_string;
use crate::bam::assignment::formatting::format_decimal;
use crate::bam::assignment::formatting::format_key_value_contract;
use crate::bam::BamCorpusFixtureValidationReport;
use crate::damage::BamDamageFixtureValidationReport;
use anyhow::Result;
pub(in crate::bam::assignment) struct EvidenceProfiles {
    pub(in crate::bam::assignment) damage_expectation: String,
    pub(in crate::bam::assignment) damage_assets: String,
    pub(in crate::bam::assignment) inherited_adna_profile: String,
}
pub(in crate::bam::assignment) fn build_profiles(
    damage_fixture: &BamDamageFixtureValidationReport,
    adna_bam_fixture: &BamCorpusFixtureValidationReport,
) -> Result<EvidenceProfiles> {
    let damage_expectation = format_key_value_contract([
        ("ct5p", format_decimal(damage_fixture.expected_damage.terminal_c_to_t_5p)),
        ("ga3p", format_decimal(damage_fixture.expected_damage.terminal_g_to_a_3p)),
        ("signal", damage_fixture.expected_damage.damage_signal.clone()),
        ("short_frag", format_decimal(damage_fixture.expected_damage.short_fragment_fraction)),
        (
            "strict_profile_upgraded",
            damage_fixture.expected_damage.strict_profile_upgraded.to_string(),
        ),
        ("terminal", damage_fixture.expected_terminal_pattern_class.clone()),
        ("udg", damage_fixture.udg_model.clone()),
    ]);
    let damage_assets = format_key_value_contract([
        ("expected_damage", file_name_string(&damage_fixture.expected_damage_path)?),
        ("reference_fasta", file_name_string(&damage_fixture.reference_fasta)?),
    ]);
    let inherited_adna_profile = format_key_value_contract([
        ("signal", adna_bam_fixture.damage_signal.clone().unwrap_or_default()),
        ("terminal", adna_bam_fixture.expected_terminal_pattern_class.clone().unwrap_or_default()),
        ("udg", adna_bam_fixture.udg_model.clone().unwrap_or_default()),
    ]);

    Ok(EvidenceProfiles { damage_expectation, damage_assets, inherited_adna_profile })
}
