use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;

use crate::DefaultParams;

pub(super) fn fastq_default_rationales(
    params: &BTreeMap<StageId, DefaultParams>,
) -> BTreeMap<StageId, String> {
    let mut rationales = BTreeMap::new();
    for stage_id in params.keys() {
        rationales.entry(stage_id.clone()).or_insert_with(|| "pipeline default".to_string());
    }
    rationales
}
