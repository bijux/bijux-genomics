//! Owner: bijux-dna-bench-corpus
//! DAMAGE corpus manifest ownership.
use super::{
    anyhow, fs, BamDamageFixtureExpectation, BamDamageFixtureManifest, Path, Result,
    BAM_DAMAGE_FIXTURE_EXPECTATION_SCHEMA_VERSION, BAM_DAMAGE_FIXTURE_SCHEMA_VERSION,
};
use anyhow::Context;

pub(super) fn load_bam_damage_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<BamDamageFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_bam_damage_fixture_expectation_path(
    expected_damage_path: &Path,
) -> Result<BamDamageFixtureExpectation> {
    let raw = fs::read_to_string(expected_damage_path)
        .with_context(|| format!("read {}", expected_damage_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_damage_path.display()))
}

pub(super) fn validate_bam_damage_fixture_manifest_contract(
    manifest: &BamDamageFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != BAM_DAMAGE_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!("unsupported BAM damage fixture schema `{}`", manifest.schema_version));
    }
    if manifest.fixture_id.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `fixture_id`"));
    }
    if manifest.sample_id.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `sample_id`"));
    }
    if manifest.species.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `species`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `description`"));
    }
    if manifest.udg_model.trim().is_empty() {
        return Err(anyhow!("BAM damage fixture must declare a non-empty `udg_model`"));
    }
    if manifest.expected_terminal_pattern_class.trim().is_empty() {
        return Err(anyhow!(
            "BAM damage fixture must declare a non-empty `expected_terminal_pattern_class`"
        ));
    }
    if manifest.limitations.is_empty()
        || manifest.limitations.iter().any(|entry| entry.trim().is_empty())
    {
        return Err(anyhow!(
            "BAM damage fixture must declare at least one non-empty `limitations` entry"
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!("BAM damage fixture must declare at least one `source_paths` entry"));
    }
    Ok(())
}

pub(super) fn validate_bam_damage_fixture_expectation_contract(
    expectation: &BamDamageFixtureExpectation,
) -> Result<()> {
    if expectation.schema_version != BAM_DAMAGE_FIXTURE_EXPECTATION_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported BAM damage expectation schema `{}`",
            expectation.schema_version
        ));
    }
    if expectation.fixture_id.trim().is_empty() {
        return Err(anyhow!("BAM damage expectation must declare a non-empty `fixture_id`"));
    }
    if expectation.sample_id.trim().is_empty() {
        return Err(anyhow!("BAM damage expectation must declare a non-empty `sample_id`"));
    }
    if !(0.0..=1.0).contains(&expectation.terminal_c_to_t_5p) {
        return Err(anyhow!("BAM damage expectation must keep `terminal_c_to_t_5p` within [0, 1]"));
    }
    if !(0.0..=1.0).contains(&expectation.terminal_g_to_a_3p) {
        return Err(anyhow!("BAM damage expectation must keep `terminal_g_to_a_3p` within [0, 1]"));
    }
    if !(0.0..=1.0).contains(&expectation.short_fragment_fraction) {
        return Err(anyhow!(
            "BAM damage expectation must keep `short_fragment_fraction` within [0, 1]"
        ));
    }
    if expectation.damage_signal.trim().is_empty() {
        return Err(anyhow!("BAM damage expectation must declare a non-empty `damage_signal`"));
    }
    Ok(())
}
