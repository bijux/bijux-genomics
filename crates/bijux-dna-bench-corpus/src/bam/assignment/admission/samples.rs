use crate::bam::BamCorpusFixtureValidationReport;
use crate::damage::BamDamageFixtureValidationReport;
use anyhow::anyhow;
use anyhow::Result;
pub(in crate::bam::assignment) fn ensure_stage_sample_matches_damage_fixture(
    stage_id: &str,
    sample_id: &str,
    fixture: &BamDamageFixtureValidationReport,
) -> Result<()> {
    if sample_id != fixture.sample_id {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{sample_id}` must match damage fixture sample `{}`",
            fixture.sample_id
        ));
    }
    Ok(())
}
pub(in crate::bam::assignment) fn ensure_stage_sample_matches_bam_fixture(
    stage_id: &str,
    sample_id: &str,
    fixture: &BamCorpusFixtureValidationReport,
) -> Result<()> {
    if fixture.samples.iter().any(|sample| sample.sample_id == sample_id) {
        return Ok(());
    }
    Err(anyhow!(
        "BAM stage `{stage_id}` sample `{sample_id}` is missing from BAM fixture `{}`",
        fixture.corpus_id
    ))
}
pub(in crate::bam::assignment) fn sample_alignment_path<'a>(
    fixture: &'a BamCorpusFixtureValidationReport,
    sample_id: &str,
) -> Result<&'a str> {
    fixture
        .samples
        .iter()
        .find(|sample| sample.sample_id == sample_id)
        .map(|sample| sample.alignment_path.as_str())
        .ok_or_else(|| anyhow!("fixture `{}` is missing sample `{sample_id}`", fixture.corpus_id))
}
