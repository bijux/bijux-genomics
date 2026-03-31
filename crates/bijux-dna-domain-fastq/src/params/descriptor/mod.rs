mod edna;
mod processing;
mod quality;

use bijux_dna_core::ids::StageId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageParamDescriptor {
    pub param_type_id: &'static str,
    pub schema_version: &'static str,
}

#[must_use]
pub fn stage_param_descriptor(stage_id: &StageId) -> Option<StageParamDescriptor> {
    quality::STAGE_PARAM_DESCRIPTORS
        .iter()
        .chain(processing::STAGE_PARAM_DESCRIPTORS.iter())
        .chain(edna::STAGE_PARAM_DESCRIPTORS.iter())
        .find_map(|(candidate_stage_id, descriptor)| {
            if stage_id == *candidate_stage_id {
                Some(*descriptor)
            } else {
                None
            }
        })
}
