use anyhow::{ensure, Result};
use std::collections::BTreeSet;
#[derive(Debug, Clone)]
pub struct FastqServingRow {
    pub stage_id: String,
    pub tool_id: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
}
pub struct FastqServingSnapshot {
    pub stage_count: usize,
    pub tool_count: usize,
    pub rows: Vec<FastqServingRow>,
}
pub type FastqAdapterCoverage =
    (usize, usize, Vec<super::adapters::FastqCommandAdapterCoverageRow>);
#[derive(Debug, Clone)]
pub struct FastqScopeRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
}
pub(super) fn admit_rows<'a>(
    stage_count: usize,
    tool_count: usize,
    rows: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<()> {
    let mut stages = BTreeSet::new();
    let mut tools = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for (stage, tool) in rows {
        ensure!(
            !stage.is_empty() && stage.trim() == stage && !tool.is_empty() && tool.trim() == tool,
            "FASTQ readiness requires nonempty unpadded identities"
        );
        ensure!(
            identities.insert((stage, tool)),
            "duplicate FASTQ readiness binding {stage}/{tool}"
        );
        stages.insert(stage);
        tools.insert(tool);
    }
    ensure!(
        stages.len() == stage_count && tools.len() == tool_count,
        "FASTQ readiness declared cardinalities do not match observed bindings"
    );
    Ok(())
}
