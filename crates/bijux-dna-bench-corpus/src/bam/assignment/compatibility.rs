use crate::bam::assignment::ports::StageCompatibility;
use crate::bam::assignment::AssignmentContext;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_core::id_catalog::BAM_PREFIX;
use std::collections::BTreeMap;
pub(in crate::bam::assignment) fn load_bam_stage_compatibility(
    context: &AssignmentContext<'_>,
) -> Result<BTreeMap<String, StageCompatibility>> {
    let rows = context.compatibility.compatibility(context.repo_root)?;
    let mut indexed = BTreeMap::new();
    for row in rows.into_iter().filter(|row| row.stage_id.starts_with(BAM_PREFIX)) {
        let stage = row.stage_id.clone();
        if indexed.insert(stage.clone(), row).is_some() {
            return Err(anyhow!("BAM corpus compatibility repeats stage `{stage}`"));
        }
    }
    Ok(indexed)
}
